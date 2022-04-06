use anyhow::{bail, Result};
use env_logger::Builder;
use log::LevelFilter;

/// Initialize logging.
pub fn init(level: &str) -> Result<()> {
    let level = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" | "warning" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        s => bail!("invalid log level: {}", s),
    };

    Builder::new().filter(Some("dotf"), level).init();
    Ok(())
}
