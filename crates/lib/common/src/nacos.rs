use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use log::{error, info, warn};
use nacos_sdk::api::config::ConfigResponse;
use nacos_sdk::api::naming::{NamingChangeEvent, NamingEventListener};
use nacos_sdk::api::{
    config::{ConfigChangeListener, ConfigService, ConfigServiceBuilder},
    naming::{NamingService, NamingServiceBuilder, ServiceInstance},
    props::ClientProps,
};
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use toml;

use crate::config::NacosConfig;
use crate::error::AppError;

struct SimpleConfigChangeListener;

impl ConfigChangeListener for SimpleConfigChangeListener {
    fn notify(&self, config_resp: ConfigResponse) {
        tracing::info!("listen the config={}", config_resp);
    }
}

pub struct SimpleInstanceChangeListener;

impl NamingEventListener for SimpleInstanceChangeListener {
    fn event(&self, event: std::sync::Arc<NamingChangeEvent>) {
        tracing::info!("subscriber notify: {:?}", event);
    }
}

pub struct NacosClient {
    config_service: Option<Box<dyn ConfigService>>,
    naming_service: Option<Box<dyn NamingService>>,
    config: NacosConfig,
    instance: Option<ServiceInstance>,
}

impl NacosClient {
    pub async fn new(config: NacosConfig) -> Result<Self, AppError> {
        let client_props = ClientProps::new()
            .server_addr(&config.server_addr)
            .namespace(&config.namespace)
            .app_name(&config.appname);

        // 添加认证信息
        let client_props = client_props
            .auth_username(&config.username)
            .auth_password(&config.password);

        // 创建配置服务
        let config_service = ConfigServiceBuilder::new(client_props.clone())
            .build()
            .map_err(|e| {
                AppError::NacosError(format!("Failed to create Nacos config client: {}", e))
            })?;

        // 创建命名服务
        let naming_service = NamingServiceBuilder::new(client_props)
            .build()
            .map_err(|e| {
                AppError::NacosError(format!("Failed to create Nacos naming client: {}", e))
            })?;

        let mut client = Self {
            config_service: Some(Box::new(config_service)),
            naming_service: Some(Box::new(naming_service)),
            config,
            instance: None,
        };

        // Fetch config immediately after initialization
        // client.get_config("", "").await?;

        Ok(client)
    }

    pub async fn listen_config<T, F>(&self, callback: F) -> Result<(), AppError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
        F: Fn(T) -> Result<(), AppError> + Send + Sync + 'static,
    {
        if let Some(config_service) = &self.config_service {
            let data_id = self.config.data_id.clone();
            let group = self.config.group.clone();

            // 自定义配置变更监听器
            let listener = Arc::new(ConfigCallbackHandler {
                callback: Arc::new(callback),
                _phantom: std::marker::PhantomData,
            });

            // 添加监听器
            config_service
                .add_listener(data_id.clone(), group.clone(), listener)
                .await
                .map_err(|e| {
                    AppError::NacosError(format!("Failed to add config listener: {}", e))
                })?;

            info!("Started listening for config changes for {}", data_id);
            Ok(())
        } else {
            Err(AppError::NacosError(
                "Config service not initialized".to_string(),
            ))
        }
    }

    // 注册服务实例
    pub async fn register_service(
        &mut self,
        service_name: &str,
        ip: &str,
        port: i32,
    ) -> Result<(), AppError> {
        if let Some(naming_service) = &self.naming_service {
            let instance = ServiceInstance {
                instance_id: None,
                service_name: Some(service_name.to_string()),
                ip: ip.to_string(),
                port,
                cluster_name: Some("DEFAULT".to_string()),
                weight: 1.0,
                healthy: true,
                enabled: true,
                ephemeral: true,
                metadata: Default::default(),
            };

            naming_service
                .register_instance(service_name.to_string(), None, instance.clone())
                .await
                .map_err(|e| AppError::NacosError(format!("Failed to register service: {}", e)))?;

            self.instance = Some(instance);
            warn!("✅ 服务注册成功: {}", service_name);
            Ok(())
        } else {
            Err(AppError::NacosError(
                "Naming service not initialized".to_string(),
            ))
        }
    }

    // 注销服务实例
    pub async fn deregister_service(&self, service_name: &str) -> Result<(), AppError> {
        if let (Some(naming_service), Some(instance)) = (&self.naming_service, &self.instance) {
            naming_service
                .deregister_instance(service_name.to_string(), None, instance.clone())
                .await
                .map_err(|e| {
                    AppError::NacosError(format!("Failed to deregister service: {}", e))
                })?;

            warn!("🛑 服务注销成功: {}", service_name);
            Ok(())
        } else {
            Err(AppError::NacosError(
                "Naming service not initialized or instance not registered".to_string(),
            ))
        }
    }

    pub async fn get_config<T: DeserializeOwned>(&self) -> Result<T, AppError> {
        if let Some(config_service) = &self.config_service {
            // 打印请求参数
            info!(
                "Requesting Nacos config - Data ID: {}, Group: {}, Namespace: {}",
                self.config.data_id, self.config.group, self.config.namespace
            );

            // 使用辅助方法获取配置内容
            let content = self.get_raw_config(&self.config.data_id, &self.config.group).await?;

            // 尝试解析为TOML
            match toml::from_str::<T>(&content) {
                Ok(config) => {
                    info!("Successfully parsed Nacos config as TOML");
                    return Ok(config);
                }
                Err(e) => {
                    error!("TOML parse error: {:?}", e);
                    error!("Content attempted to parse: {}", content);
                    
                    // 尝试进一步处理，去除可能的外部包装
                    if content.starts_with('[') && content.contains(']') {
                        // 可能是有效的TOML，但有外部文本
                        // 尝试寻找第一个[section]作为开始
                        if let Some(section_start) = content.find('[') {
                            let clean_content = &content[section_start..];
                            info!("Attempting to parse cleaned content: '{}'", clean_content);
                            
                            match toml::from_str::<T>(clean_content) {
                                Ok(config) => {
                                    info!("Successfully parsed cleaned Nacos config as TOML");
                                    return Ok(config);
                                }
                                Err(e2) => {
                                    error!("TOML parse error (second attempt): {:?}", e2);
                                }
                            }
                        }
                    }
                    
                    return Err(AppError::NacosError(format!("Failed to parse config: {}", e)));
                }
            }
        } else {
            Err(AppError::NacosError(
                "Config service not initialized".to_string(),
            ))
        }
    }

    pub async fn get_raw_config(&self, data_id: &str, group: &str) -> Result<String, AppError> {
        info!(
            "Getting config from Nacos server: data_id={}, group={}, namespace={}",
            data_id, group, self.config.namespace
        );
        
        let client = self.config_service.as_ref().unwrap();
        
        match client.get_config(data_id.to_string(), group.to_string()).await {
            Ok(raw_response) => {
                let raw_content = raw_response.content();
                info!("Raw response from Nacos: '{}'", raw_content);
                
                if raw_content.is_empty() {
                    error!("Empty response from Nacos config server");
                    return Err(AppError::NacosError(format!(
                        "Empty response when getting config for data_id={}, group={}",
                        data_id, group
                    )));
                }
                
                // 提取真正的配置内容
                let content = if let Some(content_start) = raw_content.find("content=") {
                    let content_part = &raw_content[content_start + 8..]; // 8 是 "content=" 的长度
                    info!("Extracted content part: '{}'", content_part);
                    content_part.to_string()
                } else {
                    // 如果找不到content=标记，使用整个响应
                    info!("Could not find content marker in response, using raw response");
                    raw_content.to_string()
                };
                
                Ok(content)
            }
            Err(e) => {
                error!("Failed to get config from Nacos: {}", e);
                Err(AppError::NacosError(format!(
                    "Failed to get config for data_id={}, group={}: {}",
                    data_id, group, e
                )))
            }
        }
    }
}

pub type SharedNacosClient = Arc<RwLock<NacosClient>>;

pub async fn init_nacos(config: NacosConfig) -> Result<SharedNacosClient, AppError> {
    let client = NacosClient::new(config).await?;
    Ok(Arc::new(RwLock::new(client)))
}

// 实现用于处理配置回调的结构体
struct ConfigCallbackHandler<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T) -> Result<(), AppError> + Send + Sync + 'static,
{
    callback: Arc<F>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> ConfigChangeListener for ConfigCallbackHandler<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: Fn(T) -> Result<(), AppError> + Send + Sync + 'static,
{
    fn notify(&self, config_resp: ConfigResponse) {
        let raw_content = config_resp.content();
        info!("Config change notification received: '{}'", raw_content);
        
        // 提取内容部分
        let content = if let Some(content_start) = raw_content.find("content=") {
            let content_part = &raw_content[content_start + 8..]; // 8 是 "content=" 的长度
            info!("Extracted content part: '{}'", content_part);
            content_part
        } else {
            // 如果找不到content=标记，使用整个响应
            warn!("Could not find content marker in response, using raw response");
            raw_content
        };
        
        // 尝试解析为TOML
        let toml_result = toml::from_str::<T>(content);
        if let Ok(config) = toml_result {
            info!("Successfully parsed updated config as TOML");
            if let Err(e) = (self.callback)(config) {
                error!("Error in config callback: {}", e);
            }
            return;
        }
        
        // 尝试进一步处理，去除可能的外部包装
        if content.starts_with('[') && content.contains(']') {
            // 可能是有效的TOML，但有外部文本
            if let Some(section_start) = content.find('[') {
                let clean_content = &content[section_start..];
                info!("Attempting to parse cleaned content: '{}'", clean_content);
                
                if let Ok(config) = toml::from_str::<T>(clean_content) {
                    info!("Successfully parsed cleaned updated config as TOML");
                    if let Err(e) = (self.callback)(config) {
                        error!("Error in config callback: {}", e);
                    }
                    return;
                }
            }
        }
        
        // 解析失败
        error!("Failed to parse updated config content");
        error!("TOML parse error: {:?}", toml_result.err().unwrap());
        error!("Content attempted to parse: {}", content);
    }
}
