use std::sync::Arc;
use std::fs;
use std::path::Path;
use std::marker::PhantomData;

use config::{Config, ConfigError, Environment, File};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use log::{info, error, warn};
use toml;
use serde::Serialize;

use crate::nacos::init_nacos;
use crate::error::AppError;

// 定义配置特性，所有配置结构体应实现此特性
pub trait ConfigComponent: DeserializeOwned + Serialize + Clone + Send + Sync + 'static {}

// 为所有满足约束的类型自动实现ConfigComponent特性
impl<T> ConfigComponent for T where T: DeserializeOwned + Serialize + Clone + Send + Sync + 'static {}

// 基础组件配置

/// 服务器配置
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
}

/// Nacos配置
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct NacosConfig {
    pub server_addr: String,
    pub namespace: String,
    pub group: String,
    pub data_id: String,
    pub username: String,
    pub password: String,
    pub appname: String,
}
impl NacosConfig {
    pub fn new(service_name: &str) -> Self {
        NacosConfig {
            server_addr: std::env::var("NACOS_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8848".to_string()),
            namespace: std::env::var("NACOS_NAMESPACE").unwrap_or_else(|_| "public".to_string()),
            group: std::env::var("NACOS_GROUP").unwrap_or_else(|_| "DEFAULT_GROUP".to_string()),
            data_id: service_name.to_string(),
            username: std::env::var("NACOS_USERNAME").unwrap_or_else(|_| "nacos".to_string()),
            password: std::env::var("NACOS_PASSWORD").unwrap_or_else(|_| "nacos".to_string()),
            appname: service_name.to_string(),
        }
    }
}

/// 数据库配置
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

/// Redis配置
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    pub db: Option<i32>,
}

/// 日志配置
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LogConfig {
    pub level: String,
    pub file: Option<String>,
}

/// 通用的应用配置管理器
#[derive(Debug)]
pub struct ConfigManager<T>
where
    T: ConfigComponent,
{
    pub config: Arc<T>,
    pub nacos_config: Option<NacosConfig>,
    _phantom: PhantomData<T>,
}

impl<T> ConfigManager<T>
where
    T: ConfigComponent,
{
    /// 从Nacos加载配置
    pub async fn from_nacos(nacos_config: NacosConfig) -> Result<Self, AppError> {
        info!("Loading configuration from Nacos: {}", nacos_config.data_id);
        
        // 初始化Nacos客户端
        let nacos_client = init_nacos(nacos_config.clone()).await?;
        let client = nacos_client.read().await;
        
        // 获取配置内容
        let config_content = client.get_raw_config(&nacos_config.data_id, &nacos_config.group).await?;
        
        // 解析配置
        match toml::from_str::<T>(&config_content) {
            Ok(config) => {
                info!("Successfully parsed configuration from Nacos");
                Ok(Self {
                    config: Arc::new(config),
                    nacos_config: Some(nacos_config),
                    _phantom: PhantomData,
                })
            },
            Err(e) => {
                error!("Failed to parse configuration from Nacos: {}", e);
                Err(AppError::ConfigError(format!("Failed to parse configuration: {}", e)))
            }
        }
    }
    
    /// 从文件加载配置
    pub fn from_file(file_path: &str) -> Result<Self, AppError> {
        info!("Loading configuration from file: {}", file_path);
        
        let config_content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => return Err(AppError::ConfigError(format!("Failed to read config file: {}", e))),
        };
        
        match toml::from_str::<T>(&config_content) {
            Ok(config) => {
                info!("Successfully parsed configuration from file");
                Ok(Self {
                    config: Arc::new(config),
                    nacos_config: None,
                    _phantom: PhantomData,
                })
            },
            Err(e) => {
                error!("Failed to parse configuration from file: {}", e);
                Err(AppError::ConfigError(format!("Failed to parse configuration: {}", e)))
            }
        }
    }
    
    /// 从环境变量加载配置
    pub fn from_env(prefix: &str) -> Result<Self, AppError> {
        info!("Loading configuration from environment with prefix: {}", prefix);
        
        let builder = Config::builder()
            .add_source(Environment::with_prefix(prefix).separator("__"));
            
        match builder.build() {
            Ok(config) => {
                match config.try_deserialize::<T>() {
                    Ok(config_data) => {
                        info!("Successfully parsed configuration from environment");
                        Ok(Self {
                            config: Arc::new(config_data),
                            nacos_config: None,
                            _phantom: PhantomData,
                        })
                    },
                    Err(e) => {
                        error!("Failed to deserialize configuration from environment: {}", e);
                        Err(AppError::ConfigError(format!("Failed to deserialize config: {}", e)))
                    }
                }
            },
            Err(e) => {
                error!("Failed to build configuration from environment: {}", e);
                Err(AppError::ConfigError(format!("Failed to build config: {}", e)))
            }
        }
    }
    
    /// 生成默认配置并保存到文件
    pub fn generate_default(config: T, file_path: &str) -> Result<Self, AppError> {
        info!("Generating default configuration to file: {}", file_path);
        
        let config_content = match toml::to_string_pretty(&config) {
            Ok(content) => content,
            Err(e) => return Err(AppError::ConfigError(format!("Failed to serialize config: {}", e))),
        };
        
        // 确保目录存在
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return Err(AppError::ConfigError(format!("Failed to create directory: {}", e)));
                }
            }
        }
        
        // 写入文件
        if let Err(e) = fs::write(file_path, config_content.clone()) {
            return Err(AppError::ConfigError(format!("Failed to write config file: {}", e)));
        }
        
        info!("Default configuration has been saved to {}", file_path);
        info!("Configuration content:\n{}", config_content);
        
        Ok(Self {
            config: Arc::new(config),
            nacos_config: None,
            _phantom: PhantomData,
        })
    }
    
    /// 获取配置的共享引用
    pub fn get_config(&self) -> Arc<T> {
        self.config.clone()
    }
}

// 旧的类型别名保持向后兼容性
pub type SharedConfig<T> = Arc<T>; 