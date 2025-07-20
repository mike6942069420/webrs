/********* main.rs *********/
pub const MAIN_HOST: ([u8; 4], u16) = if cfg!(debug_assertions) {
    ([127, 0, 0, 1], 8080)
} else {
    ([0, 0, 0, 0], 8080) // 0.0.0.0 because inside Docker container
};

/********* log.rs *********/
pub const LOG_FILE: &str = "data/log.txt";

/********* db.rs *********/
pub const DB_FILE: &str = "data/db.txt";
pub const DB_WRITE_INTERVAL: u64 = 1; // seconds

// 1000*200*4 bytes = 781.25 kB upfront memory allocation
pub const DB_INIT_NB_MSG: usize = 1000; // initial capacity for messages
pub const DB_MAX_MSG_SIZE: usize = 4 * 200; // max size of each message (in bytes, *4 for UTF-8 encoding)

/********* ws.rs *********/
// Maximum number of messages a clients channel can hold
pub const WS_BUFF_MESSAGES: usize = 32;
// Ping interval in seconds
pub const WS_PING_INTERVAL: u64 = if cfg!(debug_assertions) { 5 } else { 60 };
// Maximum number of users allowed in the WebSocket hub
pub const WS_MAX_USERS: usize = if cfg!(debug_assertions) { 2 } else { 100 };
