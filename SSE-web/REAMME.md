### 这个实现提供了以下功能：
- /events 端点 (GET): 客户端可以通过这个端点订阅 SSE 事件
- /broadcast 端点 (POST): 用于广播消息给所有连接的客户端

### 你可以这样测试这个服务器：
- 启动服务器：`cargo run`
- 在一个终端中使用 curl 订阅事件：`curl http://localhost:8080/events`
- 在另一个终端中发送广播消息：
`curl -X POST http://localhost:8080/broadcast -d "Hello, World!"`
- 所有订阅的客户端都会收到广播的消息。

### 这个实现使用了：
- broadcast 通道来实现消息广播
- BroadcastStream 来将接收器转换为流
- parking_lot::RwLock 提供更好的并发性能
- Actix-web 的异步处理能力

每个连接到 /events 的客户端都会获得一个新的广播接收器，并且会收到之后的所有广播消息。当客户端断开连接时，接收器会自动清理。