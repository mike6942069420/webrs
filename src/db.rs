/*  TODO: can be improved
    - Might look into more efficient data structures for messages
    - May want to store messages in json to add metadata
    - Lock times of GLOBAL_MESSAGES may be too long in render or initialize functions
*/
use crate::constants;
use once_cell::sync::Lazy;
use sailfish::{RenderError, TemplateSimple};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::error;
#[cfg(debug_assertions)]
use tracing::info;

#[derive(TemplateSimple)]
#[template(path = "../target/user_dir/index.html")] // pre-templated by build.rs
#[template(rm_whitespace = true)]
struct Template<'a> {
    pub nbusers: &'a usize,
    pub nonce: &'a str,
    pub messages: &'a Vec<String>,
}

static GLOBAL_MESSAGES: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| {
    let mut vec = Vec::with_capacity(constants::DB_INIT_NB_MSG);
    for _ in 0..constants::DB_INIT_NB_MSG {
        vec.push(String::with_capacity(constants::DB_MAX_MSG_SIZE));
    }
    Arc::new(RwLock::new(vec))
});

pub async fn add_message(msg: String) {
    let mut messages = GLOBAL_MESSAGES.write().await;
    messages.push(msg);
}

// TODO: optimize by not having to do a deep copy of the template each time we return the result
// rather render once to a buffer allocated in the calling function
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

pub async fn initialize() -> Result<(), std::io::Error> {
    // can be used by one task only
    let mut count_prev;

    // read from file and initialize GLOBAL_MESSAGES
    {
        let mut messages = GLOBAL_MESSAGES.write().await;
        messages.clear();

        // read from file
        let contents = fs::read_to_string(constants::DB_FILE).await?;
        for line in contents.lines() {
            messages.push(line.to_string());
        }

        count_prev = messages.len();
    }

    // spawn task to write to DB_FILE every 1 second
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                constants::DB_WRITE_INTERVAL,
            ))
            .await;

            let messages = GLOBAL_MESSAGES.read().await;
            let count_current = messages.len();
            if count_prev < count_current {
                let new_messages = &messages[count_prev..count_current];

                // Open the file in append mode
                if let Ok(mut file) = OpenOptions::new()
                    .append(true)
                    .open(constants::DB_FILE)
                    .await
                {
                    let buffer = new_messages.join("\n") + "\n";
                    if let Err(e) = file.write_all(buffer.as_bytes()).await {
                        error!("[D] Failed to write messages to file: {}", e);
                    }
                    #[cfg(debug_assertions)]
                    info!("[D] Wrote {} messages to file", count_current - count_prev);

                    count_prev = count_current;
                } else {
                    error!(
                        "[D] Failed to open file for writing: {}",
                        constants::DB_FILE
                    );
                }
            } else {
                #[cfg(debug_assertions)]
                info!(
                    "[D] No new messages to write to file, current count: {}",
                    count_current
                );
            }
        }
    });
    Ok(())
}
