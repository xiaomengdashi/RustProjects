
这个示例包含以下功能：
1. 定义了多个 API 端点：
- GET /users - 获取所有用户
- GET /users/{id} - 根据 ID 获取用户
- POST /users - 创建新用户
- GET /health - 健康检查端点
2. 使用 Swagger UI 提供 API 文档
- 可以通过访问 http://localhost:8080/swagger-ui/ 查看
3. 包含了完整的 API 文档注释
- 使用 utoipa 宏来生成 OpenAPI 文档
- 为每个端点添加了描述、请求参数、响应类型等信息
4. 使用 serde 进行序列化和反序列化
要运行这个程序：`cargo run`
保存文件后运行：
在浏览器中访问： `http://localhost:8080/swagger-ui/`
你将看到一个完整的 Swagger UI 界面，可以：
浏览所有可用的 API 端点
查看请求/响应模型
直接在界面上测试 API
