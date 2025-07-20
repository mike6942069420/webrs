use crate::constants;
use std::fs::OpenOptions;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;

pub fn init_logging() -> Result<WorkerGuard, std::io::Error> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(constants::LOG_FILE)?;

    let (non_blocking, guard) = non_blocking(file);

    fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .init();

    Ok(guard)
}
