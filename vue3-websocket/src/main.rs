use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("WebSocket server running on {}", addr);

    // 创建广播通道
    let (tx, _rx) = broadcast::channel::<String>(100); // 最多支持100条消息
    let tx = Arc::new(Mutex::new(tx));

    while let Ok((stream, _)) = listener.accept().await {
        let tx = Arc::clone(&tx);
        tokio::spawn(handle_connection(stream, tx));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, tx: Arc<Mutex<broadcast::Sender<String>>>) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during WebSocket handshake");

    println!("New WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    // 每个连接都会从广播通道中接收消息
    let mut rx = tx.lock().unwrap().subscribe();

    // 启动一个任务来处理广播
    tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            // 将收到的消息广播给客户端
            write.send(Message::Text(message)).await.unwrap();
        }
    });

    // 处理接收到的消息并广播
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    println!("Received: {}", text);
                    // 广播收到的消息
                    tx.lock().unwrap().send(text).unwrap();
                }
            }
            Err(e) => {
                println!("Error while processing message: {:?}", e);
                break;
            }
        }
    }
}