use std::sync::Arc;

use common::error::AppError;
use common::model::User;
use common::response::{res_json_custom, res_json_err, res_json_ok, Res};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use crate::service::user_service::UserService;

// 服务上下文，保存全局服务实例
lazy_static::lazy_static! {
    static ref USER_SERVICE: tokio::sync::RwLock<Option<Arc<UserService>>> = tokio::sync::RwLock::new(None);
}

// 初始化服务
pub async fn init_service(service: Arc<UserService>) {
    let mut writer = USER_SERVICE.write().await;
    *writer = Some(service);
}

// 获取路由
pub fn routes() -> Router {
    Router::new()
        .push(Router::with_path("users")
            .get(get_users)
            .post(create_user))
        .push(Router::with_path("users/<id>")
            .get(get_user)
            .delete(delete_user))
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<User>,
}

/// 创建用户
#[endpoint(
    tags("用户"),
    responses(
        (status_code = 200, description = "创建用户成功"),
        (status_code = 400, description = "请求参数错误"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn create_user(body: JsonBody<CreateUserRequest>) -> Res<UserResponse> {
    let service = USER_SERVICE.read().await;
    let service = service.as_ref().expect("UserService not initialized");
    
    let user_req = body.into_inner();
    
    match service.create_user(user_req.username, user_req.email).await {
        Ok(created_user) => {
            let resp = UserResponse { user: created_user };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::BadRequest(msg) => Err(res_json_custom(400, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
}

/// 获取指定用户
#[endpoint(
    tags("用户"),
    responses(
        (status_code = 200, description = "获取用户成功"),
        (status_code = 404, description = "用户不存在"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_user(req: &mut Request) -> Res<UserResponse> {
    let service = USER_SERVICE.read().await;
    let service = service.as_ref().expect("UserService not initialized");
    
    let id = req.param::<String>("id").unwrap_or_default();
    
    match service.get_user(&id).await {
        Ok(user) => {
            let resp = UserResponse { user };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
}

/// 获取用户列表
#[endpoint(
    tags("用户"),
    responses(
        (status_code = 200, description = "获取用户列表成功"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn get_users(_req: &mut Request) -> Res<UserListResponse> {
    let service = USER_SERVICE.read().await;
    let service = service.as_ref().expect("UserService not initialized");
    
    match service.get_users().await {
        Ok(users) => {
            let resp = UserListResponse { users };
            Ok(res_json_ok(Some(resp)))
        }
        Err(e) => Err(res_json_err(e.to_string())),
    }
}

/// 删除用户
#[endpoint(
    tags("用户"),
    responses(
        (status_code = 200, description = "删除用户成功"),
        (status_code = 404, description = "用户不存在"),
        (status_code = 500, description = "服务器内部错误")
    ),
)]
async fn delete_user(req: &mut Request) -> Res<()> {
    let service = USER_SERVICE.read().await;
    let service = service.as_ref().expect("UserService not initialized");
    
    let id = req.param::<String>("id").unwrap_or_default();
    
    match service.delete_user(&id).await {
        Ok(_) => Ok(res_json_ok(None)),
        Err(e) => match e {
            AppError::NotFound(msg) => Err(res_json_custom(404, msg)),
            _ => Err(res_json_err(e.to_string())),
        }
    }
} 