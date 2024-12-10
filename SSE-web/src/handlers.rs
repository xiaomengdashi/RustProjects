use actix_web::{web, HttpResponse};
use futures::StreamExt;
use std::sync::Arc;
use std::pin::Pin;
use futures_util::stream::Stream;
use tokio_stream::wrappers::BroadcastStream;
use crate::state::AppState;

pub async fn sse_handler(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|msg| {
            match msg {
                Ok(msg) => {
                    let id = format!("id: {}\n", msg.0);
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