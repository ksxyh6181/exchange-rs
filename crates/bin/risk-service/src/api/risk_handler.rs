use std::sync::Arc;

use common::error::AppError;
use common::model::{RiskEvaluation, RiskLevel};
use common::response::{res_json_custom, res_json_err, res_json_ok, Res};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use crate::service::risk_service::RiskService;

// 服务上下文，保存全局服务实例
lazy_static::lazy_static! {
    static ref RISK_SERVICE: tokio::sync::RwLock<Option<Arc<RiskService>>> = tokio::sync::RwLock::new(None);
}

// 初始化服务
pub async fn init_service(service: Arc<RiskService>) {
    let mut writer = RISK_SERVICE.write().await;
    *writer = Some(service);
}

// 获取路由
pub fn routes() -> Router {
    Router::new()
        .push(Router::with_path("risk/evaluate")
            .post(evaluate_order))
        .push(Router::with_path("risk/users/<user_id>")
            .get(get_user_risk_profile))
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct EvaluateOrderRequest {
    pub user_id: String,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct RiskEvaluationResponse {
    pub approved: bool,
    pub reason: Option<String>,
    pub risk_level: Option<RiskLevel>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserRiskProfileResponse {
    pub risk_profile: RiskEvaluation,
}

/// 评估订单风险
#[endpoint(
    tags("风控"),
    responses(
        (status_code = 200, description = "风险评估成功"),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 404, description = "用户不存在"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn evaluate_order(body: JsonBody<EvaluateOrderRequest>) -> Res<RiskEvaluationResponse> {
    let service = RISK_SERVICE.read().await;
    let service = service.as_ref().expect("RiskService not initialized");
    
    let order_req = body.into_inner();
    let order_value = order_req.price * order_req.quantity;
    
    match service.evaluate_order(&order_req.user_id, order_value).await {
        Ok((approved, reason, risk_level)) => {
            let resp = RiskEvaluationResponse {
                approved,
                reason,
                risk_level,
            };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            AppError::BadRequest(msg) => Err(res_json_custom(400, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
}

/// 获取用户风险档案
#[endpoint(
    tags("风控"),
    responses(
        (status_code = 200, description = "获取风险档案成功"),
        (status_code = 404, description = "用户风险档案不存在"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_user_risk_profile(req: &mut Request) -> Res<UserRiskProfileResponse> {
    let service = RISK_SERVICE.read().await;
    let service = service.as_ref().expect("RiskService not initialized");
    
    let user_id = req.param::<String>("user_id").unwrap_or_default();
    
    match service.get_user_risk_profile(&user_id).await {
        Ok(risk_profile) => {
            let resp = UserRiskProfileResponse { risk_profile };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
} 