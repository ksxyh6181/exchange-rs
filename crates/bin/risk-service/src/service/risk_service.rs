use std::collections::HashMap;
use std::sync::RwLock;

use common::error::{AppError, AppResult};
use common::model::{RiskEvaluation, RiskLevel};

pub struct RiskService {
    risk_profiles: RwLock<HashMap<String, RiskEvaluation>>,
}

impl RiskService {
    pub fn new() -> Self {
        let mut risk_profiles = HashMap::new();
        
        // 添加一些示例风险配置
        risk_profiles.insert(
            "user1".to_string(),
            RiskEvaluation {
                user_id: "user1".to_string(),
                risk_level: RiskLevel::Low,
                max_order_value: 10000.0,
                max_daily_volume: 50000.0,
                updated_at: chrono::Utc::now().timestamp(),
            },
        );
        
        risk_profiles.insert(
            "user2".to_string(),
            RiskEvaluation {
                user_id: "user2".to_string(),
                risk_level: RiskLevel::Medium,
                max_order_value: 5000.0,
                max_daily_volume: 20000.0,
                updated_at: chrono::Utc::now().timestamp(),
            },
        );
        
        risk_profiles.insert(
            "user3".to_string(),
            RiskEvaluation {
                user_id: "user3".to_string(),
                risk_level: RiskLevel::High,
                max_order_value: 1000.0,
                max_daily_volume: 5000.0,
                updated_at: chrono::Utc::now().timestamp(),
            },
        );
        
        Self {
            risk_profiles: RwLock::new(risk_profiles),
        }
    }

    pub async fn get_user_risk_profile(&self, user_id: &str) -> AppResult<RiskEvaluation> {
        let profiles = self.risk_profiles.read().unwrap();
        
        profiles
            .get(user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Risk profile not found for user: {}", user_id)))
    }

    pub async fn evaluate_order(
        &self,
        user_id: &str,
        order_value: f64,
    ) -> AppResult<(bool, Option<String>, Option<RiskLevel>)> {
        // 获取用户风险配置
        let risk_profile = match self.get_user_risk_profile(user_id).await {
            Ok(profile) => profile,
            Err(_) => {
                // 如果用户没有风险配置，使用默认配置（中等风险级别）
                return Ok((
                    order_value <= 5000.0,
                    if order_value > 5000.0 {
                        Some("Order value exceeds default limit of 5000.0".to_string())
                    } else {
                        None
                    },
                    Some(RiskLevel::Medium),
                ));
            }
        };
        
        // 检查订单值是否超过最大限额
        if order_value > risk_profile.max_order_value {
            return Ok((
                false,
                Some(format!(
                    "Order value ({}) exceeds the maximum allowed ({}) for this user",
                    order_value, risk_profile.max_order_value
                )),
                Some(risk_profile.risk_level),
            ));
        }
        
        // 在真实系统中，这里会有更多复杂的风险检查，例如：
        // - 检查用户当日交易量是否超过限额
        // - 检查是否存在异常交易模式
        // - 检查是否来自可疑IP地址或设备
        // - 等等...
        
        // 简化示例：订单通过风险评估
        Ok((true, None, Some(risk_profile.risk_level)))
    }

    // 在真实场景中，还会有更新用户风险评级的方法、批量风险评估方法等
} 