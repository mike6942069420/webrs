use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct Message {
    pub content: String,
}

// Global messages store as a static RwLock-wrapped Vec<Message>
static GLOBAL_MESSAGES: Lazy<Arc<RwLock<Vec<Message>>>> =
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub async fn add_message(msg: Message) {
    let mut messages = GLOBAL_MESSAGES.write().await;
    messages.push(msg);
}

pub async fn get_messages() -> Vec<Message> {
    let messages = GLOBAL_MESSAGES.read().await;
    messages.clone()
}

#[cfg(not(debug_assertions))]
pub async fn init_messages() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::time::{self, Duration};
    use tokio_postgres::{NoTls};
    
    // add a 3 second delay before initializing the DB
    time::sleep(Duration::from_secs(1)).await;

    // Connect to the Postgres DB — adjust connection string as needed
    let (client, connection) =
        tokio_postgres::connect("host=postgres user=postgres password=123456789 dbname=webserver_db", NoTls).await?;

    // Spawn a task to manage the connection in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let create_table = "
        CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            content TEXT NOT NULL
        );
        ";

    client.execute(create_table, &[]).await?;

    // Load messages from the DB into GLOBAL_MESSAGES
    let rows = client.query("SELECT content FROM messages", &[]).await?;

    {
        let mut messages = GLOBAL_MESSAGES.write().await;
        messages.clear();
        for row in rows {
            let content: String = row.get("content");
            messages.push(Message { content });
        }
    }

    // Spawn a periodic task to upload messages back to the DB every 1 second
    let global_messages = GLOBAL_MESSAGES.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let messages = global_messages.read().await;
            let snapshot = messages.clone();

            // This example truncates then inserts all messages — adjust for your use case
            if let Err(e) = client.execute("TRUNCATE TABLE messages", &[]).await {
                eprintln!("Failed to truncate messages: {}", e);
                continue;
            }

            for msg in snapshot.iter() {
                if let Err(e) = client.execute("INSERT INTO messages (content) VALUES ($1)", &[&msg.content]).await {
                    eprintln!("Failed to insert message: {}", e);
                }
            }
        }
    });

    Ok(())
}