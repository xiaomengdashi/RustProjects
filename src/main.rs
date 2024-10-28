use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

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
        let tx = Arc::new(Mutex::new(tx));
        let rx: Arc<Mutex<mpsc::Receiver<(String, SocketAddr)>>> = Arc::new(Mutex::new(rx));

        // 生产者线程：监听UDP消息并发送到通道
        thread::spawn({
            let socket = socket_clone.clone();
            let tx = tx.clone();
            move || {
                let mut buf = [0; 1024];
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            let message = String::from_utf8_lossy(&buf[..size]).to_string();
                            let sender = tx.lock().unwrap();
                            if let Err(e) = sender.send((message, src_addr)) {
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

        // 消费者线程：从通道接收消息并通过UDP发送回客户端
        thread::spawn({
            let socket = socket_clone;
            let rx = rx.clone();
            move || {
                loop {
                    let receiver = rx.lock().unwrap();
                    match receiver.recv() {
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

fn main() {
    let udp_server = UdpServer::new("0.0.0.0:12345").expect("Could not start server");
    println!("udp server is running on 0.0.0.0:12345");
    udp_server.run();
    
    // 保持主线程运行，以便服务器线程继续运行
    std::thread::park();
}