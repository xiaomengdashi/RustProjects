use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        User {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            password_hash,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }

    pub fn set_password(&mut self, password: String) {
        self.password_hash = hash(password, DEFAULT_COST).unwrap();
    }
}


pub struct Database {
    users: HashMap<String, User>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            users: HashMap::new(),
        }
    }

    pub async fn create_user(&mut self, username: String, password: String) -> User {
        let user = User::new(username, password);
        self.users.insert(user.id.clone(), user.clone());
        user
    }

    pub async fn get_user(&self, username: String) -> Option<User> {
        self.users.values().find(|user| user.username == username).cloned()
    }

    pub fn get_user_mut(&mut self, username: String) -> Option<&mut User> {
        self.users.values_mut().find(|user| user.username == username)
    }

    pub async fn has_user(&self, username: String) -> bool {
        self.users.values().any(|user| user.username == username)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}