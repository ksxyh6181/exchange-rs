use std::sync::Arc;

use common::error::AppError;
use common::model::Trade;
use common::response::{res_json_custom, res_json_err, res_json_ok, Res};
use salvo::oapi::endpoint;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use crate::service::exchange_service::ExchangeService;

// 服务上下文，保存全局服务实例
lazy_static::lazy_static! {
    static ref EXCHANGE_SERVICE: tokio::sync::RwLock<Option<Arc<ExchangeService>>> = tokio::sync::RwLock::new(None);
}

// 初始化服务
pub async fn init_service(service: Arc<ExchangeService>) {
    let mut writer = EXCHANGE_SERVICE.write().await;
    *writer = Some(service);
}

// 获取路由
pub fn routes() -> Router {
    Router::new()
        .push(Router::with_path("trades")
            .get(get_trades))
        .push(Router::with_path("trades/<id>")
            .get(get_trade))
        .push(Router::with_path("symbols")
            .get(get_symbols))
        .push(Router::with_path("prices/<symbol>")
            .get(get_price))
}

#[derive(Serialize, Debug, ToSchema)]
pub struct TradeResponse {
    pub trade: Trade,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct TradeListResponse {
    pub trades: Vec<Trade>,
}

#[derive(Deserialize, Debug)]
pub struct SymbolRequest {
    pub symbol: String,
}

/// 获取交易列表
#[endpoint(
    tags("交易"),
    responses(
        (status_code = 200, description = "获取交易列表成功"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_trades(req: &mut Request) -> Res<TradeListResponse> {
    let service = EXCHANGE_SERVICE.read().await;
    let service = service.as_ref().expect("ExchangeService not initialized");
    
    let order_id = req.query::<String>("order_id").unwrap_or_default().to_string();
    let user_id = req.query::<String>("user_id").unwrap_or_default().to_string();
    
    let order_id_option = if order_id.is_empty() { None } else { Some(order_id) };
    let user_id_option = if user_id.is_empty() { None } else { Some(user_id) };
    
    match service.get_trades(order_id_option, user_id_option).await {
        Ok(trades) => {
            let resp = TradeListResponse { trades };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => Err(res_json_err(e.to_string())),
    }
}

/// 获取指定交易
#[endpoint(
    tags("交易"),
    responses(
        (status_code = 200, description = "获取交易成功"),
        (status_code = 404, description = "交易不存在"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_trade(req: &mut Request) -> Res<TradeResponse> {
    let service = EXCHANGE_SERVICE.read().await;
    let service = service.as_ref().expect("ExchangeService not initialized");
    
    let id = req.param::<String>("id").unwrap_or_default();
    
    match service.get_trade(&id).await {
        Ok(trade) => {
            let resp = TradeResponse { trade };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
}

/// 获取交易对列表
#[endpoint(
    tags("交易"),
    responses(
        (status_code = 200, description = "获取交易对列表成功")
    ),
)]
async fn get_symbols(_req: &mut Request) -> Res<serde_json::Value> {
    // 简化示例，返回固定的交易对列表
    let symbols = serde_json::json!({
        "symbols": ["BTC/USDT", "ETH/USDT", "BNB/USDT", "SOL/USDT"]
    });
    
    Ok(res_json_ok(Some(symbols)))
}

/// 获取交易对价格
#[endpoint(
    tags("交易"),
    responses(
        (status_code = 200, description = "获取价格成功"),
        (status_code = 404, description = "交易对不存在")
    ),
)]
async fn get_price(req: &mut Request) -> Res<serde_json::Value> {
    let symbol = req.param::<String>("symbol").unwrap_or_default();
    
    // 简化示例，为特定交易对返回模拟价格
    let price = match symbol.as_str() {
        "BTC/USDT" => 66500.0,
        "ETH/USDT" => 3450.0,
        "BNB/USDT" => 560.0,
        "SOL/USDT" => 142.0,
        _ => return Err(res_json_custom(404, format!("Symbol not found: {}", symbol))),
    };
    
    let response_data = serde_json::json!({
        "symbol": symbol,
        "price": price,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    Ok(res_json_ok(Some(response_data)))
} 