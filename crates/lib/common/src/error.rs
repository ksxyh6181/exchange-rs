use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Nacos error: {0}")]
    NacosError(String),
    
    #[error("Config error: {0}")]
    ConfigError(String),
}

pub type AppResult<T> = Result<T, AppError>; 