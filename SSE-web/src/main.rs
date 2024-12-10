
use SSE_web::state;
use SSE_web::server;
use SSE_web::handlers;
use SSE_web::broadcaster;

use tokio::spawn;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = state::AppState::new();
    
    // Spawn broadcaster task
    spawn(broadcaster::run_broadcaster(state.tx.clone()));

    // Run server
    server::run_server(state).await
}
