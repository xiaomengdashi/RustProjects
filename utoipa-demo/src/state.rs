use std::sync::Mutex;
use crate::models::user::User;

pub struct AppState {
    pub users: Mutex<Vec<User>>,
    pub next_id: Mutex<u32>,
} 