use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;
use tokio::sync::broadcast;


#[tokio::main]
async fn main() {
    
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    let (tx, rx) = broadcast::channel::<String>(10); 
    
    println!("WebSocket server running at {}", addr);

    while let Ok((stream, _)) = listener.accept().await {

        let tx = tx.clone();   
        let mut rx = rx.resubscribe();
        
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the WebSocket handshake");

            let (mut write,mut read) = ws_stream.split();

            loop {
                tokio::select! {
                    received_message = rx.recv() => {
                        if let Ok(received_message) = received_message {
                            write.send(Message::Text(received_message)).await.unwrap();
                        }
                    }
                    read_message = read.next() => {
                        if let Some(Ok(Message::Text(text))) = read_message {
                            tx.send(text).unwrap();
                        }
                    }
                }
            }
        });
    }
}