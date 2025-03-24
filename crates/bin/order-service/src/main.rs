mod api;
mod service;

use std::sync::Arc;

use common::config::{ConfigManager, NacosConfig, ServerConfig, DatabaseConfig};
use common::logger;
use common::nacos;
use log::{error, info, warn};
use salvo::prelude::*;
use service::order_service::OrderService;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "order_service";
const SERVICE_REGISTER_NAME: &str = "order-service";

// 定义服务特定的配置结构
#[derive(Debug, Deserialize, Clone, Serialize)]
struct OrderServiceConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    // 可以添加订单服务特有的配置
    pub order_timeout_seconds: Option<u64>,
    pub max_orders_per_user: Option<u32>,
}

#[tokio::main]
async fn main() {
    // 初始化日志
    logger::init();
    info!("Starting order service...");

    // 1. 创建并连接Nacos
    let nacos_config = NacosConfig::new(SERVICE_NAME);
    
    info!("Connecting to Nacos server at {}", nacos_config.server_addr);
    let nacos_client = match nacos::init_nacos(nacos_config.clone()).await {
        Ok(client) => {
            info!("Successfully connected to Nacos server");
            client
        },
        Err(e) => {
            error!("Failed to connect to Nacos server: {}", e);
            return;
        }
    };
    
    // 2. 从Nacos获取配置
    info!("Retrieving configuration from Nacos: {}", nacos_config.data_id);
    let config = {
        let client = nacos_client.read().await;
        let config_content = match client.get_raw_config(&nacos_config.data_id, &nacos_config.group).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to get configuration from Nacos: {}", e);
                return;
            }
        };
        
        match toml::from_str::<OrderServiceConfig>(&config_content) {
            Ok(config) => {
                info!("Successfully parsed configuration from Nacos");
                Arc::new(config)
            },
            Err(e) => {
                error!("Failed to parse configuration: {}", e);
                return;
            }
        }
    };
    
    info!("Using configuration: {:?}", config);
    
    // 3. 注册服务到Nacos
    info!("Registering service with Nacos: {}", SERVICE_REGISTER_NAME);
    if let Err(e) = nacos_client
        .write()
        .await
        .register_service(SERVICE_REGISTER_NAME, &config.server.host, config.server.port)
        .await 
    {
        error!("Failed to register service with Nacos: {}", e);
        warn!("Continuing without service registration");
    } else {
        info!("Service successfully registered with Nacos");
    }

    // 4. 创建服务并启动HTTP服务器
    let order_service = Arc::new(OrderService::new());
    api::order_handler::init_service(order_service).await;
    let router = api::order_handler::routes();

    let address = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting HTTP server on {}", address);
    Server::new(TcpListener::new(address).bind().await).serve(router).await;
}