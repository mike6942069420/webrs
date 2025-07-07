use crate::constants;

use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::{self, Message};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};
use tracing::{error, info};

type UserId = usize;
type Tx = Sender<Message>;

static GLOBAL_HUB: Lazy<Arc<RwLock<HashMap<UserId, Tx>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

static GLOBAL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

pub async fn handle_websocket(websocket: HyperWebsocket) -> Result<(), tungstenite::Error> {
    let websocket = websocket.await?;
    let (ws_sink, mut ws_stream) = websocket.split(); // Split into sink and stream

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(constants::WS_BUFF_MESSAGES);  // SEND main loop --> forward_task
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(());                               // KILL main loop --> ping_task

    let ws_sink = Arc::new(Mutex::new(ws_sink)); // goes into main loop
    let ping_sink = Arc::clone(&ws_sink); // goes into ping_task
    let forward_sink = Arc::clone(&ws_sink); // goes into forward_sink task

    // Generate a unique user ID
    let user_id = GLOBAL_ID.fetch_add(1, Ordering::Relaxed);

    // Register the user in the global hub
    {
        let mut hub = GLOBAL_HUB.write().await;
        hub.insert(user_id, tx);
    }

    // forward_task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut sink = forward_sink.lock().await;
            if let Err(e) = sink.send(msg).await {
                error!("Failed to send message to user {}: {e}", user_id);
                break;
            }
        }

        // Remove user on disconnect
        let mut hub = GLOBAL_HUB.write().await;
        hub.remove(&user_id);
        info!("User {} disconnected and removed from hub", user_id);
    });

    // ping_task
    let ping_task = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(constants::WS_PING_INTERVAL));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let mut sink = ping_sink.lock().await;
                    if let Err(e) = sink.send(Message::Ping(Vec::new().into())).await {
                        error!("Error sending ping: {e}");
                        break;
                    } else {
                        #[cfg(debug_assertions)]
                        info!("Sent ping message");
                    }
                }
                _ = shutdown_rx.changed() => {
                    break;
                }
            }
        }
    });

    // main loop
    while let Some(message) = ws_stream.next().await {
        if let Err(e) = &message {
            error!("Error receiving message: {e}");
            break;
        }

        match message? {
            Message::Binary(msg) => {
                let mut sink = ws_sink.lock().await;
                sink.send(Message::binary(b"Thank you, come again.".to_vec()))
                    .await?;
                #[cfg(debug_assertions)]
                info!("Received binary message: {:?}", msg);
                // send it to all users
            }

            Message::Close(msg) => {
                if let Some(msg) = &msg {
                    println!(
                        "Received close message with code {} and message: {}",
                        msg.code, msg.reason
                    );
                } else {
                    println!("Received close message");
                }
                break;
            }
            _ => {}
        }
    }

    // Signal ping task to stop
    let _ = shutdown_tx.send(());

    // Wait for ping task to finish
    let _ = ping_task.await;

    println!("WebSocket connection closed cleanly");
    Ok(())
}
