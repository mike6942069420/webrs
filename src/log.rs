use crate::constants;
use std::fs::OpenOptions;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(constants::LOG_FILE)
        .expect("Failed to open log file");

    let (non_blocking, guard) = non_blocking(file);

    fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .init();

    guard
}
