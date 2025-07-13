use once_cell::sync::Lazy;
use tokio::sync::RwLock;

// Global messages store as a static RwLock-wrapped Vec<Message>
static GLOBAL_MESSAGES: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(Vec::new()));

pub async fn add_message(msg: String) {
    let mut messages = GLOBAL_MESSAGES.write().await;
    messages.push(msg);
}

pub async fn get_messages() -> Vec<String> {
    let messages = GLOBAL_MESSAGES.read().await;
    messages.clone()
}
