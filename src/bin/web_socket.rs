use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;


#[tokio::main]
async fn main() {
    
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    
    println!("WebSocket server running at {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the WebSocket handshake");

            let (mut write,mut read) = ws_stream.split();

            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        println!("Received: {}", text);
                        write.send(Message::Text(format!(" {}",text))).await.unwrap();
                    }
                    Ok(_) => {
                        println!("Received a non-text message");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            };
        });
    }
}