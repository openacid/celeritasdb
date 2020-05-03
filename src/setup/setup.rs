use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use std::io::{self, Error, ErrorKind};

use slog::Drain;

use super::log_format::CeleFormat;

/// init a global log
pub fn init_logger() -> io::Result<()> {
    // TODO: add log config to init logger
    let mut log_path = match env::current_dir() {
        Ok(p) => p,
        Err(_) => PathBuf::new(),
    };
    log_path.push("cele.log");

    let file = open_log_file(log_path)?;

    let decorator = slog_term::PlainDecorator::new(file);
    let drain = CeleFormat::new(decorator).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let logger = slog::Logger::root(drain, slog::o!());

    slog_global::set_global(logger);

    info!("logger ready");
    Ok(())
}

/// Opens log file with append mode. Creates a new log file if it doesn't exist.
fn open_log_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let path = path.as_ref();
    let parent = path.parent().ok_or_else(|| {
        Error::new(
            ErrorKind::Other,
            "Unable to get parent directory of log file",
        )
    })?;
    if !parent.is_dir() {
        fs::create_dir_all(parent)?
    }
    OpenOptions::new().append(true).create(true).open(path)
}
