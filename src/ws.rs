use crate::constants;

use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::{self, Message};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug)]
struct JsonMessage {
    ty: String, // "id" "error" "local" "message" "nbusers"
    id: usize,  // id of the messages sender
    co: String, // content of the message
}

type UserId = usize;
type Tx = Sender<Message>;

static GLOBAL_HUB: Lazy<Arc<RwLock<HashMap<UserId, Tx>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

static GLOBAL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

async fn broadcast_to_all(msg: Message) {
    let hub = GLOBAL_HUB.read().await;
    for (_user_id, tx) in hub.iter() {
        // try_send(msg.clone()) or send(msg.clone()).await are to options (first is fail fast, second is blocking)
        // may want to avoid .clone() if messages are large => Use Arc::new(msg) and Arc::clone(&msg) instead
        if let Err(e) = tx.try_send(msg.clone()) {
            match e {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    // User's queue is full — drop message
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    // User has disconnected — remove from hub
                }
            }
        }
    }
}

pub async fn get_user_count() -> usize {
    let hub = GLOBAL_HUB.read().await;
    hub.len()
}

// May combine forward_task and ping_task into a single task to reduce lock contention
pub async fn handle_websocket(websocket: HyperWebsocket) -> Result<(), tungstenite::Error> {
    let websocket = websocket.await?;

    let (ws_sink, mut ws_stream) = websocket.split(); // Split into sink and stream
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(constants::WS_BUFF_MESSAGES); // SEND main loop --> forward_task
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(()); // to kill the forward_task and ping_task
    let ws_sink = Arc::new(Mutex::new(ws_sink)); // goes into main loop
    let ping_sink = Arc::clone(&ws_sink); // goes into ping_task
    let forward_sink = Arc::clone(&ws_sink); // goes into forward_task

    // Generate a unique user ID
    let user_id = GLOBAL_ID.fetch_add(1, Ordering::Relaxed);

    // Register the user in the global hub
    {
        let mut hub = GLOBAL_HUB.write().await;
        hub.insert(user_id, tx);
    }

    // forward_task
    // sends messages from the user's channel to the WebSocket sink
    let mut shutdown_rx_fwd = shutdown_rx.clone();
    let forward_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                maybe_msg = rx.recv() => {
                    match maybe_msg {
                        Some(msg) => {
                            let mut sink = forward_sink.lock().await;
                            if let Err(e) = sink.send(msg).await {
                                error!("Failed to send message to user {}: {e}", user_id);
                                break;
                            }
                        }
                        None =>  break
                    }
                }
                _ = shutdown_rx_fwd.changed() => break

            }
        }
        let mut hub = GLOBAL_HUB.write().await;
        hub.remove(&user_id);
    });

    // ping_task
    // sends periodic pings to the Websocket sink
    let mut shutdown_rx_ping = shutdown_rx.clone();
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
                _ = shutdown_rx_ping.changed() => break

            }
        }
    });

    // TODO: send initial id message to user
    //let initial_message = JsonMessage {
    //    ty: "id".to_string(),
    //    id: 0,
    //    co: user_id.to_string(),
    //};    //if let Err(e) = ws_sink.lock().await.send(Message::Text(serde_json::to_string(&initial_message).unwrap())).await {
    //    error!("Failed to send initial message to user {}: {e}", user_id);
    //    return Err(e);
    //}
    // TODO: send the Nbusers message to the client

    // main loop
    // receives messages from the WebSocket stream
    while let Some(message) = ws_stream.next().await {
        if let Err(e) = &message {
            error!("Error receiving message: {e}");
            break;
        }

        match message? {
            Message::Binary(msg) => {
                #[cfg(debug_assertions)]
                info!("Received binary message: {:?}", msg);

                broadcast_to_all(Message::Binary(msg)).await;
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

    // shutdown forward and ping tasks
    let _ = shutdown_tx.send(());
    let _ = ping_task.await;
    let _ = forward_task.await;

    println!("WebSocket connection closed cleanly");
    Ok(())
}
