use crate::color;
use anyhow::{bail, Result};
use env_logger::fmt::Formatter;
use env_logger::Builder;
use log::{LevelFilter, Record};
use std::io::Write;

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

    let mut builder = Builder::new();

    // Do not show any logs from these modules with level lower than warning
    // for m in &[""] {
    //     builder.filter_module(m, LevelFilter::Warn);
    // }

    builder.filter_level(level).format(format).init();
    Ok(())
}

fn format(f: &mut Formatter, r: &Record) -> std::io::Result<()> {
    let level = r.level();
    let clr = match level {
        log::Level::Error => color::RED,
        log::Level::Warn => color::YELLOW,
        log::Level::Info => color::GREEN,
        log::Level::Debug => color::BLUE,
        log::Level::Trace => color::CYAN,
    };
    writeln!(f, "{}{:^5}{} - {}", clr, level, color::RESET, r.args())
}
