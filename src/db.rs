use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct Message {
    pub content: String,
}

// Global messages store as a static RwLock-wrapped Vec<Message>
static GLOBAL_MESSAGES: Lazy<Arc<RwLock<Vec<Message>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub async fn add_message(msg: Message) {
    let mut messages = GLOBAL_MESSAGES.write().await;
    messages.push(msg);
}

pub async fn get_messages() -> Vec<Message> {
    let messages = GLOBAL_MESSAGES.read().await;
    messages.clone()
}
