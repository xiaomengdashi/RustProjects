use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

lazy_static::lazy_static! {
    static ref CHANNEL: (Arc<Mutex<mpsc::Sender<(String, SocketAddr)>>>, Arc<Mutex<mpsc::Receiver<(String, SocketAddr)>>>) = {
        let (tx, rx) = mpsc::channel();
        (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
    };
}

// 生产者线程：从 UDP 接收消息并发送到通道
fn udp_listener(socket: Arc<UdpSocket>) {
    let (tx, _) = CHANNEL.clone();
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

// 消费者线程：从通道接收消息并通过 UDP 发送回客户端
fn channel_listener(socket: Arc<UdpSocket>) {
    let (_, rx) = CHANNEL.clone();

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
                break; // 发生错误时退出循环
            }
        }
    }
}

fn main() {
    // 创建 UDP 套接字并绑定到本地地址
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:12345").expect("Could not bind socket"));

    // 创建生产者线程，监听 UDP 消息
    let producer_socket = socket.clone();
    let producer = thread::spawn(move || {
        udp_listener(producer_socket);
    });

    // 创建消费者线程，从通道接收消息并通过 UDP 发送回客户端
    let consumer_socket = socket.clone();
    let consumer = thread::spawn(move || {
        channel_listener(consumer_socket);
    });

    // 等待生产者和消费者线程完成
    producer.join().unwrap();
    consumer.join().unwrap();
}