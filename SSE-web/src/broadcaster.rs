use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

pub async fn run_broadcaster(tx: broadcast::Sender<(usize, String)>) {
    let mut counter = 0;
    loop {
        counter += 1;
        let data = format!("Message #{}", counter);
        
        if tx.receiver_count() > 0 {
            match tx.send((counter, data)) {
                Ok(_) => println!("Successfully broadcasted message #{}", counter),
                Err(e) => println!("Failed to broadcast message: {}", e),
            }
        } else {
            println!("No active subscribers, skipping message #{}", counter);
        }
        
        sleep(Duration::from_secs(1)).await;
    }
} 