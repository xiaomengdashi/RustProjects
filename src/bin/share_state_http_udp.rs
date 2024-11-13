use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc::{self, Sender, Receiver}, Arc, Mutex};
use std::thread;
use log::{info, error};
use actix_web::{web, App, HttpServer, Responder};

pub struct UdpServer {
    socket: Arc<UdpSocket>,
}

impl UdpServer {
    pub fn new(bind_addr: &str) -> std::io::Result<Self> {
        let socket = Arc::new(UdpSocket::bind(bind_addr)?);
        Ok(Self { socket })
    }

    pub fn run(self, shared_state: Arc<Mutex<SharedState>>) {
        let socket_clone = self.socket.clone();
        let (tx, rx) = mpsc::channel();

        self.start_producer(socket_clone.clone(), tx.clone(), shared_state.clone());
        self.start_consumer(socket_clone.clone(), rx, shared_state.clone());
    }

    fn start_producer(&self, socket: Arc<UdpSocket>, tx: Sender<(String, SocketAddr)>, shared_state: Arc<Mutex<SharedState>>) {
        thread::spawn({
            let socket = socket.clone();
            let tx = tx.clone();
            let shared_state = shared_state.clone();
            move || {
                let mut buf = [0; 1024];
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((size, src_addr)) => {
                            let message = String::from_utf8_lossy(&buf[..size]).to_string();
                            if let Err(e) = tx.send((message.clone(), src_addr)) {
                                error!("Failed to send message to channel: {}", e);
                            }
                            // Update shared state
                            {
                                let mut state = shared_state.lock().unwrap();
                                state.udp_messages.push(message);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                }
            }
        });
    }

    fn start_consumer(&self, socket: Arc<UdpSocket>, rx: Receiver<(String, SocketAddr)>, shared_state: Arc<Mutex<SharedState>>) {
        thread::spawn({
            let socket = socket.clone();
            let shared_state = shared_state.clone();
            move || {
                loop {
                    match rx.recv() {
                        Ok((message, src_addr)) => {
                            info!("Received and sending back: {}", message);
                            if let Err(e) = socket.send_to(message.as_bytes(), src_addr) {
                                error!("Failed to send message: {}", e);
                            }
                            // Update shared state
                            {
                                let mut state = shared_state.lock().unwrap();
                                state.udp_responses.push(message);
                            }
                        }
                        Err(e) => {
                            error!("Receiver error: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }
}

#[derive(Default)]
pub struct SharedState {
    pub udp_messages: Vec<String>,
    pub udp_responses: Vec<String>,
}

async fn index(data: web::Data<Arc<Mutex<SharedState>>>) -> impl Responder {
    let state = data.lock().unwrap();
    format!("UDP messages: {:?}\nUDP responses: {:?}", state.udp_messages, state.udp_responses)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Initialize shared state
    let shared_state = Arc::new(Mutex::new(SharedState::default()));

    // Start UDP server
    let udp_server = UdpServer::new("0.0.0.0:12345").expect("Could not start UDP server");
    info!("UDP server is running on 0.0.0.0:12345");
    udp_server.run(shared_state.clone());

    // Start actix-web server
    info!("Starting actix-web server on http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_state.clone()))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}