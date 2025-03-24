use std::collections::HashMap;
use std::sync::RwLock;

use common::error::{AppError, AppResult};
use common::model::Trade;
use uuid::Uuid;

pub struct ExchangeService {
    trades: RwLock<HashMap<String, Trade>>,
}

impl ExchangeService {
    pub fn new() -> Self {
        Self {
            trades: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_trade(&self, id: &str) -> AppResult<Trade> {
        let trades = self.trades.read().unwrap();
        
        trades
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Trade not found: {}", id)))
    }

    pub async fn get_trades(
        &self,
        order_id: Option<String>,
        user_id: Option<String>,
    ) -> AppResult<Vec<Trade>> {
        let trades = self.trades.read().unwrap();
        
        let filtered_trades = match (order_id, user_id) {
            (Some(order_id), _) => {
                // 按订单ID过滤
                trades
                    .values()
                    .filter(|trade| trade.order_id == order_id)
                    .cloned()
                    .collect()
            }
            (_, Some(_user_id)) => {
                // 实际场景中应该通过用户ID查询相关订单，然后获取对应的交易记录
                // 简化示例中，我们只模拟返回一个空的交易列表
                Vec::new()
            }
            _ => {
                // 返回所有交易
                trades.values().cloned().collect()
            }
        };
        
        Ok(filtered_trades)
    }

    // 实际交易引擎中，这个方法会撮合订单生成交易记录
    // 此处简化实现，仅用于演示
    pub async fn create_trade(
        &self,
        order_id: String,
        price: f64,
        quantity: f64,
    ) -> AppResult<Trade> {
        let trade = Trade {
            id: Uuid::new_v4().to_string(),
            order_id,
            price,
            quantity,
            commission: price * quantity * 0.001, // 0.1% 手续费
            created_at: chrono::Utc::now().timestamp(),
        };
        
        let mut trades = self.trades.write().unwrap();
        trades.insert(trade.id.clone(), trade.clone());
        
        Ok(trade)
    }
} 