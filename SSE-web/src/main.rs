use actix_web::{web, App, HttpResponse, HttpServer};
use futures::StreamExt;
use futures_util::stream::Stream;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

// 定义广播通道的容量
const CHANNEL_CAPACITY: usize = 100;

// 定义应用状态，包含广播发送器
struct AppState {
    tx: broadcast::Sender<(usize, String)>,
}

// SSE handler function
async fn sse_handler(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|msg| {
            match msg {
                Ok(msg) => {
                    let id = format!("id: {}\n", msg.0);  // 添加ID
                    let data = format!("data: {}\n\n", msg.1);
                    println!("Sending message to client: {}{}", id, data);
                    Ok::<_, actix_web::Error>(web::Bytes::from(format!("{}{}", id, data)))
                },
                Err(e) => {
                    println!("Error sending message: {:?}", e);
                    Ok::<_, actix_web::Error>(web::Bytes::from(""))
                }
            }
        });

    let stream: Pin<Box<dyn Stream<Item = Result<web::Bytes, actix_web::Error>>>> = Box::pin(stream);

    HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .append_header(("Access-Control-Allow-Origin", "*"))
        .streaming(stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 创建广播通道
    let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
    let state = Arc::new(AppState { tx });

    // Spawn a new thread to send messages
    let tx_clone = state.tx.clone();
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            counter += 1;
            let data = format!("Message #{}", counter);
            
            // 检查是否有活跃的接收者
            if tx_clone.receiver_count() > 0 {
                match tx_clone.send((counter, data)) {
                    Ok(_) => println!("Successfully broadcasted message #{}", counter),
                    Err(e) => println!("Failed to broadcast message: {}", e),
                }
            } else {
                println!("No active subscribers, skipping message #{}", counter);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    println!("Server running at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/events", web::get().to(sse_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
