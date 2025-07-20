use crate::constants;
use crate::db;

use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::HyperWebsocket;
use hyper_tungstenite::tungstenite::Utf8Bytes;
use hyper_tungstenite::tungstenite::{self, Message};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::Sender;
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug)]
struct JsonMessage {
    r#type: String, // "id" "error" "local" "message" "nbusers" "info"
    id: usize,      // id of the messages sender
    content: Value, // content of the message
}

type UserId = usize;
type Tx = Sender<Message>;

static GLOBAL_HUB: Lazy<DashMap<UserId, Tx>> = Lazy::new(DashMap::new);

static GLOBAL_ID: AtomicUsize = AtomicUsize::new(1); // the user ID starts at 1, 0 is server ID

macro_rules! send_message {
    ($ws_sink:expr, $msg:expr, $ip:expr) => {
        let _ = $ws_sink.lock().await.send($msg).await.map_err(|e| {
            error!("    [{}] WS: Failed to send message: {}", $ip, e);
            e
        })?;
    };
}

fn to_bytes(msg: &JsonMessage) -> Message {
    let json_string = serde_json::to_string(msg).expect("Failed to serialize message");
    let bytes = Utf8Bytes::from(json_string);
    Message::Text(bytes)
}

fn broadcast_to_all(msg: Message) -> Result<(), ()> {
    let mut success = true;

    for entry in GLOBAL_HUB.iter() {
        let tx = entry.value();
        if let Err(e) = tx.try_send(msg.clone()) {
            success = false;

            match e {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    // TODO: send disconnect message to user
                    // maybe log it or count it as recoverable
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    // TODO: remove user from hub
                }
            }
        }
    }

    if success { Ok(()) } else { Err(()) }
}

#[inline(always)]
pub fn get_user_count() -> usize {
    GLOBAL_HUB.len()
}

// INFO: May combine forward_task and ping_task into a single task to reduce lock contention
pub async fn handle_websocket(
    websocket: HyperWebsocket,
    ip: IpAddr,
) -> Result<(), tungstenite::Error> {
    let mut websocket = websocket.await?;

    // max user count check
    if get_user_count() >= constants::WS_MAX_USERS {
        error!(
            "   [{}] WS: Maximum number of users reached: {}",
            ip,
            constants::WS_MAX_USERS
        );
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
    let user_id = GLOBAL_ID.fetch_add(1, Ordering::Relaxed); // TODO: handle overflow

    // Register the user in the global hub
    GLOBAL_HUB.insert(user_id, tx);

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
                                error!("    [{}] WS [{}]: Failed to send message to user: {}", ip, user_id, e);
                                break;
                            }
                        }
                        None =>  break
                    }
                }
                _ = shutdown_rx_fwd.changed() => break
            }
        }
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
                        error!("    [{}] WS [{}]: Failed to send ping: {}", ip, user_id, e);
                        break;
                    } else {
                        #[cfg(debug_assertions)]
                        info!("    [{}] WS [{}]: Sent ping to user", ip, user_id);
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
    send_message!(ws_sink, initial_message, ip);

    let nb_users = get_user_count();

    // send initial user count
    let user_count_message = to_bytes(&JsonMessage {
        r#type: "nbusers".to_string(),
        id: 0,
        content: nb_users.to_string().into(),
    });
    let _ = broadcast_to_all(user_count_message); // TODO: handle error

    info!(
        "    [{}] WS [{}]: New user connected, {} users",
        ip, user_id, nb_users
    );

    // main loop
    // receives messages from the WebSocket stream
    while let Some(message) = ws_stream.next().await {
        if let Err(e) = &message {
            error!(
                "    [{}] WS [{}]: Error receiving message: {}",
                ip, user_id, e
            );
            break;
        }

        match message? {
            Message::Text(msg) => {
                info!("    [{}] WS [{}]: Received Text: {}", ip, user_id, msg);

                // if the message type is message, broadcast it to all users
                if let Ok(json_msg) = serde_json::from_str::<JsonMessage>(&msg) {
                    match json_msg.r#type.as_str() {
                        "message" => {
                            // TODO: validate message content and length

                            // Broadcast message to all users
                            if broadcast_to_all(Message::Text(msg)).is_err() {
                                error!(
                                    "    [{}] WS [{}]: Failed to broadcast message: {}",
                                    ip, user_id, json_msg.content
                                );

                                // TODO: make message static
                                let error_message = to_bytes(&JsonMessage {
                                    r#type: "error".to_string(),
                                    id: 0,
                                    content: "Internal server error, please try again later."
                                        .into(),
                                });
                                send_message!(ws_sink, error_message, ip);
                            } else {
                                info!(
                                    "    [{}] WS [{}]: Broadcasted message: {}",
                                    ip, user_id, json_msg.content
                                );

                                // Store the message in the database
                                db::add_message(json_msg.content.as_str().unwrap().to_string())
                                    .await;
                            }
                        }
                        "info" => {
                            // receive info message
                        }

                        "local" => {
                            // receive local message
                        }

                        _ => {
                            error!(
                                "    [{}] WS [{}]: Unknown message type: {}",
                                ip, user_id, json_msg.r#type
                            );
                            break;
                        }
                    }
                } else {
                    error!(
                        "    [{}] WS [{}]: Failed to deserialize message: {}",
                        ip, user_id, msg
                    );
                    break;
                }
            }

            Message::Binary(msg) => {
                error!("    [{}] WS [{}]: Received Binary: {:?}", ip, user_id, msg);
                break;
            }

            Message::Close(_) => {
                break;
            }

            _ => {}
        }
    }

    // shutdown forward and ping tasks and remove user from hub
    let _ = shutdown_tx.send(());
    let _ = ping_task.await;
    let _ = forward_task.await;
    GLOBAL_HUB.remove(&user_id);

    let nb_users = get_user_count();

    // update user count
    let user_count_message = to_bytes(&JsonMessage {
        r#type: "nbusers".to_string(),
        id: 0,
        content: nb_users.to_string().into(),
    });
    let _ = broadcast_to_all(user_count_message); // TODO: handle error

    info!(
        "    [{}] WS [{}]: User disconnected, {} users left",
        ip, user_id, nb_users
    );

    Ok(())
}
