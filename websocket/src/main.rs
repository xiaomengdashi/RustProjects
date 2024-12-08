use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::Message;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;
use tokio::time::{interval, Duration};
use chrono;

// 定义广播器类型
type ClientMap = Arc<Mutex<HashMap<String, actix_ws::Session>>>;

// WebSocket 处理函数
async fn ws_handler(
    req: HttpRequest, 
    stream: web::Payload,
    clients: web::Data<ClientMap>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;
    let client_id = Uuid::new_v4().to_string();
    
    // 将新客户端添加到连接列表
    clients.lock().await.insert(client_id.clone(), session.clone());
    println!("新客户端连接: {}", client_id);
    
    // 在新的任务中处理 websocket 会话
    let clients_clone = clients.clone();
    actix_web::rt::spawn(async move {
        let ws = session;
        
        // WebSocket 消息处理循环
        let mut msg_stream = msg_stream;
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Text(text) => {
                    // 广播消息给所有客户端
                    let mut clients = clients_clone.lock().await;
                    let broadcast_msg = format!("客户端 {} 说: {}", client_id, text);
                    
                    // 创建要移除的客户端ID列表
                    let mut disconnected_clients = Vec::new();
                    
                    // 向所有客户端发送消息
                    for (id, client) in clients.iter_mut() {
                        if let Err(_) = client.text(&broadcast_msg).await {
                            disconnected_clients.push(id.clone());
                        }
                    }
                    
                    // 移除断开连接的客户端
                    for id in disconnected_clients {
                        clients.remove(&id);
                        println!("客户端断开连接: {}", id);
                    }
                }
                Message::Close(reason) => {
                    let _ = ws.close(reason).await;
                    // 从连接列表中移除客户端
                    clients_clone.lock().await.remove(&client_id);
                    println!("客户端主动断开连接: {}", client_id);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(res)
}

// 定期广播数据的任务
async fn broadcast_periodic_data(clients: web::Data<ClientMap>) {
    let mut interval = interval(Duration::from_secs(1));  // 每秒执行一次
    let mut counter = 0;

    loop {
        interval.tick().await;
        counter += 1;
        
        let mut clients = clients.lock().await;
        if clients.is_empty() {
            continue;
        }

        let message = format!("服务器定时消息 #{}: 当前时间 {:?}", 
            counter, 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let mut disconnected_clients = Vec::new();
        
        for (id, client) in clients.iter_mut() {
            if let Err(_) = client.text(&message).await {
                disconnected_clients.push(id.clone());
            }
        }

        for id in disconnected_clients {
            clients.remove(&id);
            println!("定时广播时发现断开的客户端: {}", id);
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("WebSocket 服务器启动在 127.0.0.1:8080");
    
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let clients_for_broadcast = web::Data::new(clients.clone());

    // 启动定期广播任务
    actix_web::rt::spawn(broadcast_periodic_data(clients_for_broadcast));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(clients.clone()))
            .route("/ws", web::get().to(ws_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
