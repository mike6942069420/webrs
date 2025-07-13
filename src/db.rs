/*  TODO: can be greatly optimized
    - Write to file only when messages change
    - Write to file with tokio::fs::write
    - Track deltas instead of full messages
    - Use a more efficient data structure for messages (pre allocation)
*/
use crate::constants;
use once_cell::sync::Lazy;
use sailfish::{RenderError, TemplateSimple};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

#[derive(TemplateSimple)]
#[template(path = "index.html")]
struct Template<'a> {
    pub nbusers: &'a usize,
    pub nonce: &'a str,
    pub messages: &'a Vec<String>,
}

static GLOBAL_MESSAGES: Lazy<Arc<RwLock<Vec<String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub async fn add_message(msg: String) {
    let mut messages = GLOBAL_MESSAGES.write().await;
    messages.push(msg);
}

pub async fn render(nbusers: &usize, nonce: &str) -> Result<String, RenderError> {
    // might lock a bit too long but does not copy
    let store = GLOBAL_MESSAGES.clone();
    let messages = store.read().await;

    // does a deep copy but the lock is shorter, would not need ARC either
    //let messages = {
    //    let guard = GLOBAL_MESSAGES.read().await;
    //    guard.clone() // fast and avoids holding lock during render
    //};

    let template = Template {
        nbusers,
        nonce,
        messages: &messages,
    };

    template.render_once()
}

pub async fn initialize() {
    {
        let mut messages = GLOBAL_MESSAGES.write().await;
        messages.clear();
        // read from file
        if let Ok(lines) = std::fs::read_to_string(constants::DB_FILE) {
            for line in lines.lines() {
                messages.push(line.to_string());
            }
        }
    }

    // spawn task to write to DB_FILE every 1 second
    let store = GLOBAL_MESSAGES.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                constants::DB_WRITE_INTERVAL,
            ))
            .await;
            let messages = store.read().await;
            let content = messages.join("\n");
            if let Err(e) = std::fs::write(constants::DB_FILE, content) {
                error!("DDD Failed to write messages to file: {}", e);
            }
        }
    });
}
