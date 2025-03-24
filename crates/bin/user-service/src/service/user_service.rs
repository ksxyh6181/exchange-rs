use std::collections::HashMap;
use std::sync::RwLock;

use common::error::{AppError, AppResult};
use common::model::User;
use uuid::Uuid;

pub struct UserService {
    users: RwLock<HashMap<String, User>>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_user(&self, username: String, email: String) -> AppResult<User> {
        // 简单校验
        if username.is_empty() {
            return Err(AppError::BadRequest("Username cannot be empty".to_string()));
        }
        if email.is_empty() {
            return Err(AppError::BadRequest("Email cannot be empty".to_string()));
        }

        // 检查用户名是否已存在
        let users = self.users.read().unwrap();
        if users.values().any(|u| u.username == username) {
            return Err(AppError::BadRequest(format!("Username '{}' already exists", username)));
        }
        
        // 创建用户
        let user = User {
            id: Uuid::new_v4().to_string(),
            username,
            email,
        };
        
        // 保存用户
        let mut users = self.users.write().unwrap();
        users.insert(user.id.clone(), user.clone());
        
        Ok(user)
    }

    pub async fn get_user(&self, id: &str) -> AppResult<User> {
        let users = self.users.read().unwrap();
        
        users
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("User not found: {}", id)))
    }

    pub async fn get_users(&self) -> AppResult<Vec<User>> {
        let users = self.users.read().unwrap();
        let user_list = users.values().cloned().collect();
        
        Ok(user_list)
    }

    pub async fn delete_user(&self, id: &str) -> AppResult<()> {
        // 检查用户是否存在
        {
            let users = self.users.read().unwrap();
            if !users.contains_key(id) {
                return Err(AppError::NotFound(format!("User not found: {}", id)));
            }
        }
        
        // 删除用户
        let mut users = self.users.write().unwrap();
        users.remove(id);
        
        Ok(())
    }
} 