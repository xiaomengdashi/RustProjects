use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::sync::Arc;
use actix_web::{web, App, HttpServer, Responder};

pub struct UdpServer {
    socket: Arc<UdpSocket>,
}

impl UdpServer {
    // 创建新的 UDP 回声服务器
    pub async fn new(bind_addr: &str) -> std::io::Result<Self> {
        let socket = Arc::new(UdpSocket::bind(bind_addr).await?);
        Ok(Self { socket })
    }

    // 启动服务器
    pub async fn run(self) {
        let socket_clone = self.socket.clone();
        let (tx, rx) = mpsc::channel::<(String, std::net::SocketAddr)>(100); // 使用带缓冲的通道

        // 启动生产者任务
        let producer = self.start_producer(socket_clone.clone(), tx);
        // 启动消费者任务
        let consumer = self.start_consumer(socket_clone.clone(), rx);

        // 同时运行生产者和消费者
        tokio::join!(producer, consumer);
    }

    // 生产者任务：监听UDP消息并发送到通道
    async fn start_producer(
        &self,
        socket: Arc<UdpSocket>,
        tx: mpsc::Sender<(String, std::net::SocketAddr)>,
    ) {
        let mut buf = [0; 1024];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, src_addr)) => {
                    let message = String::from_utf8_lossy(&buf[..size]).to_string();
                    if let Err(e) = tx.send((message, src_addr)).await {
                        eprintln!("Failed to send message to channel: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {}", e);
                }
            }
        }
    }

    // 消费者任务：从通道接收消息并通过UDP发送回客户端
    async fn start_consumer(
        &self,
        socket: Arc<UdpSocket>,
        mut rx: mpsc::Receiver<(String, std::net::SocketAddr)>,
    ) {
        while let Some((message, src_addr)) = rx.recv().await {
            println!("Received and sending back: {}", message);
            if let Err(e) = socket.send_to(message.as_bytes(), src_addr).await {
                eprintln!("Failed to send message: {}", e);
            }
        }
    }
}

async fn index() -> impl Responder {
    "Hello, world from actix-web!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Start UDP server
    let udp_server = UdpServer::new("0.0.0.0:12345").await.expect("Could not start UDP server");
    println!("UDP server is running on 0.0.0.0:12345");
    
    // 使用 tokio spawn 来运行 UDP 服务器
    tokio::spawn(async move {
        udp_server.run().await;
    });

    // Start actix-web server
    println!("Starting actix-web server on http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}