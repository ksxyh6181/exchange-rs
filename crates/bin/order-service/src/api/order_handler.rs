use std::sync::Arc;
use anyhow::Error;
use salvo::oapi::extract::JsonBody;
use common::error::AppError;
use common::model::{Order, OrderSide, OrderStatus, OrderType};
use common::response::{res_json_custom, res_json_err, res_json_ok, Res};

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::order_service::OrderService;

// 服务上下文，保存全局服务实例
lazy_static::lazy_static! {
    static ref ORDER_SERVICE: tokio::sync::RwLock<Option<Arc<OrderService>>> = tokio::sync::RwLock::new(None);
}

// 初始化服务
pub async fn init_service(service: Arc<OrderService>) {
    let mut writer = ORDER_SERVICE.write().await;
    *writer = Some(service);
}

// 获取路由
pub fn routes() -> Router {
    Router::new()
        .push(Router::with_path("orders")
            .get(get_orders)
            .post(create_order))
        .push(Router::with_path("orders/<id>")
            .get(get_order)
            .delete(cancel_order))
}

#[derive(Deserialize, Debug,ToSchema)]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct OrderResponse {
    pub order: Order,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct OrderListResponse {
    pub orders: Vec<Order>,
}

/// 创建订单
#[endpoint(
    tags("订单"),
    responses(
        (status_code = 200, description = "创建订单成功"),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn create_order(body: JsonBody<CreateOrderRequest>) -> Res<OrderResponse> {
    let service = ORDER_SERVICE.read().await;
    let service = service.as_ref().expect("OrderService not initialized");
    
    let order_req = body.into_inner();
    let timestamp = chrono::Utc::now().timestamp();
    
    let order = Order {
        id: Uuid::new_v4().to_string(),
        user_id: order_req.user_id,
        symbol: order_req.symbol,
        order_type: order_req.order_type,
        side: order_req.side,
        price: order_req.price,
        quantity: order_req.quantity,
        status: OrderStatus::New,
        created_at: timestamp,
        updated_at: timestamp,
    };

    match service.create_order(order).await {
        Ok(created_order) => {
            let resp = OrderResponse { order: created_order };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::BadRequest(msg) => Err(res_json_custom(400, msg)),
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
}

/// 获取指定订单
#[endpoint(
    tags("订单"),
    responses(
        (status_code = 200, description = "获取订单成功"),
        (status_code = 404, description = "订单不存在"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_order(req: &mut Request) -> Res<OrderResponse> {
    let service = ORDER_SERVICE.read().await;
    let service = service.as_ref().expect("OrderService not initialized");
    
    let id = req.param::<String>("id").unwrap_or_default();
    
    match service.get_order(&id).await {
        Ok(order) => {
            let resp = OrderResponse { order };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
}

/// 获取订单列表
#[endpoint(
    tags("订单"),
    responses(
        (status_code = 200, description = "获取订单列表成功"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_orders(req: &mut Request) -> Res<OrderListResponse> {
    let service = ORDER_SERVICE.read().await;
    let service = service.as_ref().expect("OrderService not initialized");
    
    let user_id = req.query::<String>("user_id").unwrap_or_default().to_string();
    let user_id_option = if user_id.is_empty() { None } else { Some(&user_id) };
    
    match service.get_orders(user_id_option).await {
        Ok(orders) => {
            let resp = OrderListResponse { orders };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => Err(res_json_err(e.to_string())),
    }
}

/// 取消订单
#[endpoint(
    tags("订单"),
    responses(
        (status_code = 200, description = "取消订单成功"),
        (status_code = 404, description = "订单不存在"),
        (status_code = 400, description = "订单不能取消"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn cancel_order(req: &mut Request) -> Res<OrderResponse> {
    let service = ORDER_SERVICE.read().await;
    let service = service.as_ref().expect("OrderService not initialized");
    
    let id = req.param::<String>("id").unwrap_or_default();
    
    match service.cancel_order(&id).await {
        Ok(order) => {
            let resp = OrderResponse { order };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            AppError::BadRequest(msg) => Err(res_json_custom(400, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
} 