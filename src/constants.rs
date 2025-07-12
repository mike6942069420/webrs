pub const MAIN_HOST: ([u8; 4], u16) = ([127, 0, 0, 1], 8080);

pub const WS_BUFF_MESSAGES: usize = 32; // Maximum number of messages a clients channel can hold

#[cfg(debug_assertions)]
pub const WS_PING_INTERVAL: u64 = 5; // Ping interval in seconds
#[cfg(not(debug_assertions))]
pub const WS_PING_INTERVAL: u64 = 60; // Ping interval in seconds

#[cfg(debug_assertions)]
pub const WS_MAX_USERS: usize = 2; // Maximum number of users allowed in the WebSocket hub
#[cfg(not(debug_assertions))]
pub const WS_MAX_USERS: usize = 100; // Maximum number of users allowed in the WebSocket hub
