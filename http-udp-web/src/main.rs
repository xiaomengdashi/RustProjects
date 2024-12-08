use actix_web::{web, App, HttpResponse, HttpServer, error::ResponseError};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use std::fmt;
use uuid;
use std::collections::HashMap;

// 自定义错误类型
#[derive(Debug, Clone)]
enum ServerError {
    UdpSendError(String),
    UdpReceiveError(String),
    TimeoutError(String),
    InternalError(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::UdpSendError(msg) => write!(f, "UDP发送错误: {}", msg),
            ServerError::UdpReceiveError(msg) => write!(f, "UDP接收错误: {}", msg),
            ServerError::TimeoutError(msg) => write!(f, "超时错误: {}", msg),
            ServerError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::UdpSendError(_) => {
                HttpResponse::ServiceUnavailable().json(self.to_string())
            }
            ServerError::UdpReceiveError(_) => {
                HttpResponse::ServiceUnavailable().json(self.to_string())
            }
            ServerError::TimeoutError(_) => {
                HttpResponse::RequestTimeout().json(self.to_string())
            }
            ServerError::InternalError(_) => {
                HttpResponse::InternalServerError().json(self.to_string())
            }
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ServerError::UdpSendError(_) => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            ServerError::UdpReceiveError(_) => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            ServerError::TimeoutError(_) => actix_web::http::StatusCode::REQUEST_TIMEOUT,
            ServerError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    header: MessageHeader,
    body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MessageHeader {
    msg_type: String,
    route: String,
    request_id: String,
}

// 修改响应通道结构
#[derive(Clone)]
struct ResponseChannel {
    responses: Arc<Mutex<HashMap<String, mpsc::Sender<Result<String, ServerError>>>>>,
}

// 添加新的消息通道类型用于UDP发送
#[derive(Clone)]
struct UdpSendChannel {
    socket: Arc<UdpSocket>,
    target_addr: String,
}

async fn udp_sender(
    mut rx: mpsc::Receiver<Message>,
    udp_send_channel: UdpSendChannel,
) {
    while let Some(msg) = rx.recv().await {
        println!("处理请求 - 路由: {}", msg.header.route);
        let json = serde_json::to_string(&msg).unwrap();
        
        match udp_send_channel.socket.send_to(json.as_bytes(), &udp_send_channel.target_addr) {
            Ok(_) => println!("发送UDP请求成功"),
            Err(e) => eprintln!("UDP发送错误: {}", e),
        }
    }
}

async fn udp_receiver(
    udp_send_channel: UdpSendChannel,
    resp_channel: Arc<Mutex<ResponseChannel>>,
) {
    let mut buf = [0; 1024];
    
    loop {
        match udp_send_channel.socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                let response = String::from_utf8_lossy(&buf[..size]).to_string();
                let msg: Message = match serde_json::from_str(&response) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("解析响应消息失败: {}", e);
                        continue;
                    }
                };

                let request_id = msg.header.request_id.clone();
                
                match msg.header.msg_type.as_str() {
                    "complete" => {
                        println!("收到Complete响应: {} (ID: {})", response, request_id);
                        // Create a new scope to ensure the lock is dropped
                        {
                            let responses = resp_channel.lock().await;
                            let responses_map = responses.responses.lock().await;
                            if let Some(tx) = responses_map.get(&request_id) {
                                let tx = tx.clone();
                                drop(responses_map); // Drop the lock before the async operation
                                if let Err(e) = tx.send(Ok(msg.body)).await {
                                    eprintln!("发送响应失败: {}", e);
                                }
                            }
                        }
                    }
                    "ack" => {
                        println!("收到ACK响应: {} (ID: {})", response, request_id);
                    }
                    _ => {
                        eprintln!("收到未知类型的响应");
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }
                eprintln!("UDP接收错误: {}", e);
                
                let error = ServerError::UdpReceiveError("UDP接收失败".to_string());
                
                // 获取所有活跃的响应通道
                let responses = resp_channel.lock().await;
                let mut channels = responses.responses.lock().await;
                
                // Create error sending tasks
                let mut tasks = Vec::new();
                for (_, tx) in channels.iter() {
                    let error = error.clone();
                    let tx = tx.clone();
                    tasks.push(tokio::spawn(async move {
                        let _ = tx.send(Err(error)).await;
                    }));
                }
                
                // Clear the channels
                channels.clear();
                
                // 等待所有错误发送完成
                for task in tasks {
                    let _ = task.await;
                }
            }
        }
    }
}

// HTTP处理函数
async fn handle_request(
    route: web::Path<String>,
    tx: web::Data<mpsc::Sender<Message>>,
    resp_channel: web::Data<Arc<Mutex<ResponseChannel>>>,
) -> Result<HttpResponse, ServerError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    
    let msg = Message {
        header: MessageHeader {
            msg_type: "request".to_string(),
            route: route.into_inner(),
            request_id: request_id.clone(),
        },
        body: "".to_string(),
    };

    println!("发送请求: {} (ID: {})", msg.header.route, request_id);

    // 为这个特定请求创建响应通道
    let (response_tx, mut response_rx) = mpsc::channel::<Result<String, ServerError>>(1);
    {
        let resp_channel = resp_channel.lock().await;
        resp_channel.responses.lock().await.insert(request_id.clone(), response_tx);
    }

    // 发送消息给UDP线程
    tx.send(msg).await.map_err(|e| ServerError::InternalError(e.to_string()))?;

    // 等待响应，带超时
    let result = match timeout(Duration::from_secs(5), response_rx.recv()).await {
        Ok(Some(Ok(response))) => Ok(HttpResponse::Ok().body(response)),
        Ok(Some(Err(e))) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
        Ok(None) => Ok(HttpResponse::InternalServerError().body("未收到响应")),
        Err(_) => Ok(HttpResponse::RequestTimeout().body("请求超时")),
    };

    // 清理响应通道
    resp_channel.lock().await.responses.lock().await.remove(&request_id);
    
    result
}

// 主UDP服务器函数
async fn udp_server(rx: mpsc::Receiver<Message>, resp_channel: Arc<Mutex<ResponseChannel>>) {
    let socket = match UdpSocket::bind("127.0.0.1:8888") {
        Ok(s) => {
            // 设置非阻塞模式
            s.set_nonblocking(true).expect("设置非阻塞模式失败");
            Arc::new(s)
        }
        Err(e) => {
            eprintln!("UDP绑定失败: {}", e);
            return;
        }
    };

    let target_addr = "127.0.0.1:8889".to_string();
    println!("UDP服务器运行在 127.0.0.1:8888");

    let udp_send_channel = UdpSendChannel {
        socket: socket.clone(),
        target_addr: target_addr,
    };

    // 启动发送者线程
    let sender_channel = udp_send_channel.clone();
    let sender = tokio::spawn(async move {
        udp_sender(rx, sender_channel).await;
    });

    // 启动接收者线程
    let receiver = tokio::spawn(async move {
        udp_receiver(udp_send_channel, resp_channel).await;
    });

    // 等待两个任务完成（实际上它们会一直运行）
    let _ = tokio::try_join!(sender, receiver);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 创建通道
    let (tx, rx) = mpsc::channel::<Message>(32);
    let resp_channel = Arc::new(Mutex::new(ResponseChannel { 
        responses: Arc::new(Mutex::new(HashMap::new())),
    }));
    let udp_resp_channel = resp_channel.clone();

    // 启动UDP服务器
    tokio::spawn(async move {
        udp_server(rx, udp_resp_channel).await;
    });

    println!("HTTP服务器运行在 127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(resp_channel.clone()))
            .route("/{route}", web::get().to(handle_request))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}