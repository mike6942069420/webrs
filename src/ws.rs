use crate::constants;

use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::{self, Message};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug)]
struct JsonMessage {
    r#type: String, // "id" "error" "local" "message" "nbusers"
    id: usize,      // id of the messages sender
    content: Value, // content of the message
}

type UserId = usize;
type Tx = Sender<Message>;

static GLOBAL_HUB: Lazy<Arc<RwLock<HashMap<UserId, Tx>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

static GLOBAL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

macro_rules! send_message {
    ($ws_sink:expr, $msg:expr) => {
        let _ = $ws_sink.lock().await.send($msg).await.map_err(|e| {
            error!("Failed to send message: {e}");
            e
        })?;
    };
}

fn to_bytes(msg: &JsonMessage) -> Message {
    let json_string = serde_json::to_string(msg).expect("Failed to serialize message");
    let bytes = Utf8Bytes::from(json_string);
    Message::Text(bytes)
}

async fn broadcast_to_all(msg: Message) {
    let hub = GLOBAL_HUB.read().await;
    for (_user_id, tx) in hub.iter() {
        // try_send(msg.clone()) or send(msg.clone()).await are to options (first is fail fast, second is blocking)
        // INFO: may want to avoid .clone() if messages are large => Use Arc::new(msg) and Arc::clone(&msg) instead
        if let Err(e) = tx.try_send(msg.clone()) {
            match e {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    // TODO: send disconnect message to user
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

// INFO: May combine forward_task and ping_task into a single task to reduce lock contention
pub async fn handle_websocket(websocket: HyperWebsocket) -> Result<(), tungstenite::Error> {
    let mut websocket = websocket.await?;

    // max user count check
    if get_user_count().await >= constants::WS_MAX_USERS {
        error!("Maximum number of users reached: {}", constants::WS_MAX_USERS);
        // send error message to user
        let error_message = to_bytes(&JsonMessage {
            r#type: "error".to_string(),
            id: 0,
            content: format!(
                "Maximum number of users reached: {}",
                constants::WS_MAX_USERS
            )
            .into(),
        });
        websocket.send(error_message).await?;
        return Ok(());
    }

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

    // Send initial id message to user
    let initial_message = to_bytes(&JsonMessage {
        r#type: "id".to_string(),
        id: 0,
        content: user_id.to_string().into(),
    });
    send_message!(ws_sink, initial_message);

    // send initial user count
    let user_count_message = to_bytes(&JsonMessage {
        r#type: "nbusers".to_string(),
        id: 0,
        content: get_user_count().await.to_string().into(),
    });
    broadcast_to_all(user_count_message).await;

    // main loop
    // receives messages from the WebSocket stream
    while let Some(message) = ws_stream.next().await {
        if let Err(e) = &message {
            error!("Error receiving message: {e}");
            break;
        }

        match message? {
            Message::Text(msg) => {
                #[cfg(debug_assertions)]
                info!("Received binary message: {:?}", msg);

                // if the message type is message, broadcast it to all users
                if let Ok(json_msg) = serde_json::from_str::<JsonMessage>(&msg) {
                    match json_msg.r#type.as_str() {
                        "message" => {
                            // Broadcast message to all users
                            broadcast_to_all(Message::Text(msg)).await;
                        }
                        "local" => {
                            // Send local message to the user
                            let local_message = to_bytes(&JsonMessage {
                                r#type: "message".to_string(),
                                id: user_id,
                                content: json_msg.content,
                            });
                            send_message!(ws_sink, local_message);
                        }
                        "error" => {
                            // Send error message to the user
                            error!("Error message from user {}: {}", user_id, json_msg.content);
                            break;
                        }

                        "info" => {
                            info!("Info message from user {}: {}", user_id, json_msg.content);
                        }

                        _ => {
                            error!("Unknown message type: {}", json_msg.r#type);
                            break;
                        }
                    }
                } else {
                    error!("Failed to deserialize binary message: {:?}", msg);
                    break;
                }
            }

            Message::Close(msg) => {
                #[cfg(debug_assertions)]
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

    let user_count_message = to_bytes(&JsonMessage {
        r#type: "nbusers".to_string(),
        id: 0,
        content: get_user_count().await.to_string().into(),
    });
    broadcast_to_all(user_count_message).await;

    #[cfg(debug_assertions)]
    println!("WebSocket connection closed cleanly");

    Ok(())
}
