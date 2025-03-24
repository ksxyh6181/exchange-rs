use env_logger::{Builder, Env};
use log::LevelFilter;

/// 初始化日志系统，设置日志级别为info
pub fn init() {
    // 创建自定义日志构建器
    let mut builder = Builder::from_env(Env::default());
    
    // 设置默认日志级别为INFO
    builder.filter_level(LevelFilter::Info);
    
    // 应用配置并初始化日志系统
    builder.init();
    
    log::info!("Logger initialized with INFO level");
}

/// 使用自定义日志级别初始化日志系统
pub fn init_with_level(level: LevelFilter) {
    // 创建自定义日志构建器
    let mut builder = Builder::from_env(Env::default());
    
    // 设置自定义日志级别
    builder.filter_level(level);
    
    // 应用配置并初始化日志系统
    builder.init();
    
    log::info!("Logger initialized with custom level: {:?}", level);
} 