# 项目使用指南

本指南将帮助您设置和运行本项目，包括Nacos配置和服务注册发现功能。

## 前提条件

1. 安装 Rust 开发环境 (rustc, cargo)
2. 安装并运行 Nacos 服务器 (2.x 版本)
3. 安装 PostgreSQL 数据库

## 安装 Nacos 服务器

### 使用 Docker 安装 (推荐)

```bash
# 拉取Nacos镜像
docker pull nacos/nacos-server:v2.2.3

# 运行Nacos服务器 (单机模式)
docker run --name nacos-server -e MODE=standalone -e JVM_XMS=512m -e JVM_XMX=512m -p 8848:8848 -p 9848:9848 -d nacos/nacos-server:v2.2.3
```

Nacos控制台将可通过 http://localhost:8848/nacos 访问 (用户名/密码: nacos/nacos)

### 直接下载安装

1. 从 [Nacos官方网站](https://nacos.io/zh-cn/docs/quick-start.html) 下载最新版本
2. 解压并进入Nacos目录
3. 运行 `bin/startup.sh -m standalone` (Linux/Mac) 或 `bin\startup.cmd -m standalone` (Windows)

## 配置 Nacos 

1. 访问 Nacos 控制台: http://127.0.0.1:8848/nacos
2. 登录 (默认用户名/密码: nacos/nacos)
3. 在配置管理 > 配置列表中，为每个服务创建配置:

   a. 点击"+"按钮创建新配置
   b. 填写 Data ID (例如 "order-service")
   c. 选择 Group (使用 "DEFAULT_GROUP")
   d. 选择格式 (JSON)
   e. 配置内容参考 `nacos_config_examples` 目录下的示例文件

服务配置示例:
```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 3000
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "username": "postgres",
    "password": "postgres",
    "database": "exchange_db"
  }
}
```

为所有服务 (order-service, exchange-service, user-service, risk-service) 创建类似配置，调整端口号和数据库名称。

## 编译项目

```bash
# 在项目根目录执行
cargo build
```

## 运行服务

在不同的终端窗口中运行各个服务:

```bash
# 运行订单服务
cargo run -p order-service

# 运行交易服务
cargo run -p exchange-service

# 运行用户服务
cargo run -p user-service

# 运行风控服务
cargo run -p risk-service
```


## 验证服务注册

1. 启动任意服务后，访问 Nacos 控制台
2. 导航到服务管理 > 服务列表
3. 您应该能看到服务已成功注册到 Nacos

## 故障排除

1. **无法连接 Nacos 服务器**
   - 检查 Nacos 服务器是否正在运行
   - 验证配置中的 `server_addr` 是否正确

2. **服务无法注册到 Nacos**
   - 检查服务日志中的错误信息
   - 确保 Nacos 命名空间与配置匹配
   - 验证服务配置中的 Nacos 参数是否正确

3. **服务无法读取 Nacos 配置**
   - 确保在 Nacos 中创建了正确的配置项
   - 检查配置的 Data ID, Group 是否与服务中的设置匹配
   - 验证配置格式是否正确 (JSON)

4. **服务间无法发现彼此**
   - 确保所有服务都已正确注册到 Nacos
   - 检查服务健康状态是否为"UP"
   - 确认使用了相同的命名空间和分组

## 参考资源

- [r-nacos 官方文档](https://r-nacos.github.io/docs/)
- [Salvo 框架文档](https://salvo.rs/)
