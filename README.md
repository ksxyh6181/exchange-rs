# RustExchange - Rust微服务与Nacos集成框架的交易系统
## ！！！仅作为项目模板，功能尚未实现！！！

RustExchange是一个基于Rust语言的微服务框架，集成了Nacos服务发现和配置中心功能，采用Salvo作为Web框架，提供了一个完整的交易系统微服务架构示例。

## 项目架构

项目采用标准的Rust工作空间结构，组织为一个核心库和多个服务二进制文件：

```
rnacos_test/
├── crates/
│   ├── lib/
│   │   └── common/           # 共享库
│   └── bin/                  # 服务二进制
│       ├── exchange-service/ # 交易服务
│       ├── order-service/    # 订单服务
│       ├── risk-service/     # 风控服务
│       └── user-service/     # 用户服务
├── Cargo.toml                # 工作空间配置
├── docker-compose.yaml       # 服务部署配置
└── docker-compose-env.yaml   # 开发环境依赖配置
```

## 核心组件

### 1. 共享库 (common)

位于`crates/lib/common`，包含所有服务共享的功能：

- **配置管理** (`config.rs`): 灵活配置系统，支持从Nacos、文件和环境变量加载配置
- **Nacos集成** (`nacos.rs`): 负责Nacos客户端初始化、配置获取、服务注册等
- **统一响应** (`response.rs`): 标准化API响应格式，提高一致性
- **错误处理** (`error.rs`): 全局错误类型定义和处理
- **数据模型** (`model.rs`): 共享业务模型定义
- **日志系统** (`logger.rs`): 统一日志配置和初始化

### 2. 微服务

每个微服务遵循相同的内部结构，包含：

```
service/
├── src/
│   ├── main.rs    # 应用入口
│   ├── api/       # API接口定义
│   └── service/   # 业务逻辑
```

各服务职责：

- **订单服务**: 处理订单的创建、查询、取消等操作
- **交易服务**: 负责交易撮合、价格查询和交易记录管理
- **用户服务**: 管理用户账户，包括创建、查询和删除用户
- **风控服务**: 执行风险评估和管理风险配置

## 技术特性

1. **Nacos集成**:
   - 服务注册与发现：服务自动向Nacos注册，并能发现其他服务
   - 配置中心：从Nacos获取配置，支持动态更新

2. **标准API设计**:
   - 所有服务使用统一的响应格式
   - 支持OpenAPI文档生成
   - 完善的错误处理和状态码映射

3. **灵活配置**:
   - 多来源配置支持（Nacos、本地文件、环境变量）
   - 每个服务可定义特定配置结构
   - 类型安全的配置处理

4. **应用生命周期**:
   - 首先连接Nacos，获取配置
   - 然后注册服务
   - 最后启动HTTP服务
   - 在服务关闭时自动注销

## 部署选项

提供多种部署方式：

1. **Docker部署**:
   ```bash
   docker-compose up -d
   ```

2. **开发环境**:
   ```bash
   docker-compose -f docker-compose-env.yaml up -d  # 启动依赖服务
   cargo run -p order-service  # 在开发环境中运行服务
   ```

3. **生产环境**:
   提供标准Dockerfile，可构建精简的生产镜像

## 使用指南

1. **配置Nacos**:
   - 确保Nacos服务器在运行
   - 在Nacos中为每个服务创建配置项
   - 配置格式为TOML，示例位于`config-example.toml`

2. **启动服务**:
   确保使用正确的启动顺序：
   ```bash
   # 各服务可在单独的终端中启动
   cargo run -p order-service
   cargo run -p exchange-service
   cargo run -p user-service
   cargo run -p risk-service
   ```

3. **API访问**:
   - 订单服务API: `http://localhost:8091/orders/...`
   - 交易服务API: `http://localhost:8092/trades/...`
   - 用户服务API: `http://localhost:8093/users/...`
   - 风控服务API: `http://localhost:8094/risk/...`

## 技术栈

- **Rust**: 核心编程语言
- **Salvo**: Web框架，负责HTTP处理和路由
- **Nacos SDK**: 服务发现和配置中心
- **Tokio**: 异步运行时
- **Serde**: 序列化和反序列化
- **Config**: 配置管理
- **TOML**: 配置文件格式
- **Log**: 日志系统

## 许可证

MIT 