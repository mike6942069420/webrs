use futures_util::{SinkExt, StreamExt};
use hyper_tungstenite::tungstenite;
use hyper_tungstenite::{HyperWebsocket, tungstenite::Message};

pub async fn handle_websocket(websocket: HyperWebsocket) -> Result<(), tungstenite::Error> {
    let mut websocket = websocket.await?;

    while let Some(message) = websocket.next().await {
        // Check if the message is an error
        if let Err(e) = message {
            eprintln!("Error receiving message: {e}");
            return Err(e);
        }
        match message? {
            Message::Text(msg) => {
                println!("Received text message: {msg}");
                websocket
                    .send(Message::text(format!("Echo: {msg}")))
                    .await?;
            }
            Message::Binary(msg) => {
                println!("Received binary message: {msg:02X?}");
                websocket
                    .send(Message::binary(b"Thank you, come again.".to_vec()))
                    .await?;
            }
            Message::Ping(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                println!("Received ping message: {msg:02X?}");
            }
            Message::Pong(msg) => {
                println!("Received pong message: {msg:02X?}");
            }
            Message::Close(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                if let Some(msg) = &msg {
                    println!(
                        "Received close message with code {} and message: {}",
                        msg.code, msg.reason
                    );
                } else {
                    println!("Received close message");
                }
            }
            Message::Frame(_msg) => {
                unreachable!();
            }
        }
    }

    Ok(())
}
