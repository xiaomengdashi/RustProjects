# Rust Algorithmic Network Protocol (mqtt)

mqtt是一个用Rust编写的网络协议实现项目，专注于MQTT 3.1.1协议的核心功能实现。该项目展示了如何在Rust中实现网络协议，同时应用了多种算法和设计模式来优化代码结构。

## mqtt

1. **协议实现**：
   - 完整的MQTT 3.1.1协议核心功能
   - CONNECT、CONNACK、PUBLISH、SUBSCRIBE等核心包类型支持
   - 数据包编码和解码机制

2. **算法应用**：
   - 可变长度编码算法（用于MQTT包长度编码）
   - 校验和计算算法
   - 数据序列化和反序列化算法
   - 广播算法（用于消息分发）
   - 包解析算法（用于正确识别和处理MQTT包）

3. **设计模式**：
   - 工厂模式（用于创建不同类型的数据包）
   - 策略模式（用于处理不同的连接状态）
   - 观察者模式（用于消息订阅和通知）
   - 并发模式（用于处理多个客户端连接）
   - 命令模式（用于封装MQTT操作）
   - 状态模式（用于管理客户端状态）

4. **Rust特性**：
   - 异步编程（Tokio运行时）
   - 内存安全和并发控制
   - 模式匹配和枚举类型
   - 智能指针和引用计数

## 项目结构

```
src/
├── main.rs         # 程序入口点
├── protocol.rs     # 协议常量和类型定义
├── packet.rs       # 数据包结构和处理逻辑
├── client.rs       # MQTT客户端实现
├── server.rs       # MQTT服务端实现
└── patterns/       # 设计模式实现
    ├── mod.rs      # 模块聚合器
    ├── commands.rs # 命令模式实现
    └── states.rs   # 状态模式实现
```

## 核心组件

### 协议模块 (protocol.rs)
定义了MQTT协议的核心常量和枚举类型，包括：
- 包类型定义（CONNECT, PUBLISH, SUBSCRIBE等）
- 连接标志位常量
- 协议版本信息

### 数据包模块 (packet.rs)
实现了MQTT数据包的结构和处理逻辑：
- ConnectPacket结构体及其实现
- 数据包编码方法
- 剩余长度计算算法
- 可变长度编码实现

### 客户端模块 (client.rs)
MQTT客户端的核心实现：
- TCP连接管理
- 消息订阅和发布
- 异步回调机制
- 并发安全的消息处理
- 持续监听服务器消息 ([start_listening](file:///e%3A/workspace/Rust/ran/src/client.rs#L207-L243)方法)
- 状态管理

### 服务端模块 (server.rs)
MQTT服务端的核心实现：
- TCP监听和连接处理
- 多客户端并发支持
- 消息广播和订阅管理
- 异步消息分发机制

### 设计模式模块 (patterns/)
实现了项目中使用的设计模式：

#### 命令模式 (commands.rs)
将MQTT操作封装为命令对象：
- ConnectCommand：连接命令
- PublishCommand：发布命令
- SubscribeCommand：订阅命令
- DisconnectCommand：断开连接命令

#### 状态模式 (states.rs)
管理MQTT客户端的不同状态：
- DisconnectedState：断开连接状态
- ConnectingState：连接中状态
- ConnectedState：已连接状态
- DisconnectingState：断开连接中状态

## 设计模式应用详解

### 1. 工厂模式
通过`ConnectPacket::new()`方法创建标准的CONNECT数据包，隐藏了复杂的初始化细节。

### 2. 策略模式
使用枚举类型`PacketType`来区分不同的数据包类型，每种类型有不同的处理策略。

### 3. 观察者模式
通过`MqttClient::on_message()`方法注册回调函数，当收到特定主题的消息时自动触发相应的处理函数。

### 4. 并发模式
使用Tokio的异步运行时和同步原语（如Mutex、broadcast channel）来处理多个客户端的并发连接和消息分发。

### 5. 代码重构与优化
通过提取公共方法（如`parse_packet`和`extract_publish_payload`）消除了代码重复，提高了代码的可维护性和可读性。

### 6. 命令模式
实现了命令模式，将MQTT操作（连接、发布、订阅等）封装为命令对象，支持操作队列和更好的扩展性。

### 7. 状态模式
实现了状态模式，管理MQTT客户端的不同状态（连接、断开连接等），并根据状态限制可执行的操作。

## 算法实现亮点

### 可变长度编码
MQTT协议使用一种特殊的可变长度编码来表示剩余长度字段，这是一种高效的编码方式，能够用最少的字节数表示较大的数值。

### 异步消息处理
利用Tokio运行时和Rust的异步特性，实现了高效的消息处理机制，能够在保持低延迟的同时处理大量并发连接。

### 广播算法
服务端使用广播算法将消息分发给所有订阅了相应主题的客户端，确保消息能够及时送达。

### 包解析算法
客户端实现了高效的MQTT包解析算法，能够正确识别和处理不同类型的MQTT包（CONNECT、PUBLISH、SUBSCRIBE等），确保了客户端与服务器之间的可靠通信。

## 日志系统

本项目集成了Rust的`log`和`env_logger`库，提供带有时间戳的日志功能。日志级别可以通过环境变量进行配置：

```bash
# 设置日志级别为info
export RUST_LOG=info
# 或者在Windows上使用set命令
set RUST_LOG=info

# 设置日志级别为debug以获取更详细的信息
export RUST_LOG=debug
# 或者在Windows上使用set命令
set RUST_LOG=debug
```

日志输出格式包含时间戳、日志级别、模块名和消息内容，便于调试和监控。

## 使用说明

现在项目包含了完整的客户端和服务端实现，可以进行本地测试：

### 启动服务端
```bash
# 设置日志级别
export RUST_LOG=info
# 或者在Windows上
set RUST_LOG=info

# 启动服务端
cargo run server
```

### 启动客户端
```bash
# 设置日志级别
export RUST_LOG=info
# 或者在Windows上
set RUST_LOG=info

# 启动客户端
cargo run client
```

### 默认运行
```bash
# 设置日志级别
export RUST_LOG=info
# 或者在Windows上
set RUST_LOG=info

# 默认运行客户端
cargo run
```

## 学习价值

本项目适合想要学习以下内容的开发者：
- Rust语言特性和最佳实践
- 网络协议实现原理
- 算法在实际项目中的应用
- 设计模式在系统设计中的运用
- 异步编程和并发控制
- 客户端-服务端架构设计

## 构建和运行

```bash
cargo build
cargo run
```

注意：由于网络依赖问题，可能需要配置适当的镜像源或代理才能成功构建项目。