use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc::{self, Sender, Receiver}, Arc};
use std::thread;
use actix_web::{web, App, HttpServer, Responder};

pub struct UdpServer {
    socket: Arc<UdpSocket>,
}

impl UdpServer {
    // 创建新的 UDP 回声服务器
    pub fn new(bind_addr: &str) -> std::io::Result<Self> {
        let socket = Arc::new(UdpSocket::bind(bind_addr)?);
        Ok(Self { socket })
    }

    // 启动服务器
    pub fn run(self) {
        let socket_clone = self.socket.clone();
        let (tx, rx) = mpsc::channel();

        // 启动生产者线程
        self.start_producer(socket_clone.clone(), tx);

        // 启动消费者线程
        self.start_consumer(socket_clone.clone(), rx);
    }

    // 生产者线程：监听UDP消息并发送到通道
    fn start_producer(&self, socket: Arc<UdpSocket>, tx: Sender<(String, SocketAddr)>) {
        thread::spawn({
            let socket = socket.clone();
            let tx = tx.clone();
            move || {
                let mut buf = [0; 1024];
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            let message = String::from_utf8_lossy(&buf[..size]).to_string();
                            if let Err(e) = tx.send((message, src_addr)) {
                                eprintln!("Failed to send message to channel: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to receive data: {}", e);
                        }
                    }
                }
            }
        });
    }

    // 消费者线程：从通道接收消息并通过UDP发送回客户端
    fn start_consumer(&self, socket: Arc<UdpSocket>, rx: Receiver<(String, SocketAddr)>) {
        thread::spawn({
            let socket = socket.clone();
            move || {
                loop {
                    match rx.recv() {
                        Ok((message, src_addr)) => {
                            println!("Received and sending back: {}", message);
                            if let Err(e) = socket.send_to(message.as_bytes(), src_addr) {
                                eprintln!("Failed to send message: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Receiver error: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }
}

async fn index() -> impl Responder {
    "Hello, world from actix-web!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Start UDP server
    let udp_server = UdpServer::new("0.0.0.0:12345").expect("Could not start UDP server");
    println!("UDP server is running on 0.0.0.0:12345");
    udp_server.run();

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