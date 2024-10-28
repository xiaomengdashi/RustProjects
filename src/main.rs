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
        let rx = Arc::new(Mutex::new(rx));

        // 启动生产者线程
        self.start_producer(socket_clone.clone(), tx.clone());

        // 启动消费者线程
        self.start_consumer(socket_clone, rx.clone());
    }

    // 生产者线程：监听UDP消息并发送到通道
    fn start_producer(&self, socket: Arc<UdpSocket>, tx: Arc<Mutex<mpsc::Sender<(String, SocketAddr)>>>) {
        thread::spawn({
            let socket = socket.clone();
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
    }

    // 消费者线程：从通道接收消息并通过UDP发送回客户端
    fn start_consumer(&self, socket: Arc<UdpSocket>, rx: Arc<Mutex<mpsc::Receiver<(String, SocketAddr)>>>) {
        thread::spawn({
            let socket = socket.clone();
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