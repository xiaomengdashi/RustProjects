### 这个实现提供了以下功能：
- /events 端点 (GET): 客户端可以通过这个端点订阅 SSE 事件
- /broadcast 端点 (POST): 用于广播消息给所有连接的客户端

### 你可以这样测试这个服务器：
- 启动服务器：`cargo run`
- 浏览器访问网址：`http://localhost:8080/events`
- 所有订阅的客户端都会收到广播的消息。

### 这个实现使用了：
- broadcast 通道来实现消息广播，工作原理是只有当有接收者（订阅者）存在时，消息才会被成功发送。如果没有接收者，send 方法会返回一个错误。
- BroadcastStream 来将接收器转换为流
- Actix-web 的异步处理能力

每个连接到 /events 的客户端都会获得一个新的广播接收器，并且会收到之后的所有广播消息。当客户端断开连接时，接收器会自动清理。


### 功能模块

- main.rs - 主入口文件
- server.rs - 服务器配置和启动
- handlers.rs - 请求处理器
- state.rs - 应用状态管理
- broadcaster.rs - 消息广播逻辑

### 运行

- 启动服务器：`cargo run`
