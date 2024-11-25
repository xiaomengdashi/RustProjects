use actix_web::{web, App, HttpServer, HttpResponse};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct AppState {
    counter: Arc<Mutex<i32>>,
}

async fn index(data: web::Data<AppState>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    format!("Request number: {}", counter)
}

fn background_thread(counter: Arc<Mutex<i32>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            let mut counter = counter.lock().unwrap();
            *counter += 10;
            println!("Background thread updated counter to: {}", *counter);
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = Arc::new(Mutex::new(0));

    // Start the background thread
    background_thread(counter.clone());

    let app_state = web::Data::new(AppState {
        counter: counter.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}