use std::collections::HashMap;
use std::sync::RwLock;

use common::error::{AppError, AppResult};
use common::model::{Order, OrderStatus};

pub struct OrderService {
    orders: RwLock<HashMap<String, Order>>,
}

impl OrderService {
    pub fn new() -> Self {
        Self {
            orders: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_order(&self, order: Order) -> AppResult<Order> {
        // 这里应该调用风控服务进行风险检查
        // 示例中简化处理，直接保存订单

        let mut orders = self.orders.write().unwrap();
        orders.insert(order.id.clone(), order.clone());
        
        // 在真实项目中，这里应该将订单发送到交易引擎处理
        Ok(order)
    }

    pub async fn get_order(&self, id: &str) -> AppResult<Order> {
        let orders = self.orders.read().unwrap();
        
        orders
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", id)))
    }

    pub async fn get_orders(&self, user_id: Option<&String>) -> AppResult<Vec<Order>> {
        let orders = self.orders.read().unwrap();
        
        let filtered_orders = if let Some(user_id) = user_id {
            orders
                .values()
                .filter(|order| order.user_id == user_id.to_string())
                .cloned()
                .collect()
        } else {
            orders.values().cloned().collect()
        };
        
        Ok(filtered_orders)
    }

    pub async fn cancel_order(&self, id: &str) -> AppResult<Order> {
        let mut orders = self.orders.write().unwrap();
        
        let order = orders
            .get_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("Order not found: {}", id)))?;
        
        if order.status == OrderStatus::Filled {
            return Err(AppError::BadRequest(
                "Cannot cancel an already filled order".to_string(),
            ));
        }
        
        if order.status == OrderStatus::Canceled {
            return Err(AppError::BadRequest(
                "Order is already canceled".to_string(),
            ));
        }
        
        order.status = OrderStatus::Canceled;
        order.updated_at = chrono::Utc::now().timestamp();
        
        Ok(order.clone())
    }
} 