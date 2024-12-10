use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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
    tx: broadcast::Sender<String>,
}

// SSE处理函数
async fn sse_handler(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|msg| {
            match msg {
                Ok(msg) => Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", msg))),
                Err(_) => Ok::<_, actix_web::Error>(web::Bytes::from(""))
            }
        });

    let stream: Pin<Box<dyn Stream<Item = Result<web::Bytes, actix_web::Error>>>> = Box::pin(stream);

    HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(stream)
}

// 广播消息的处理函数
async fn broadcast_message(
    msg: String,
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    // 发送消息到所有订阅者
    let _ = state.tx.send(msg);
    HttpResponse::Ok().body("Message broadcasted")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 创建广播通道
    let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
    let state = Arc::new(AppState { tx });

    println!("Server running at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/events", web::get().to(sse_handler))
            .route("/broadcast", web::post().to(broadcast_message))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
