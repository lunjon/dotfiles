pub mod cli;
pub mod cmd;
pub mod color;
pub mod data;
pub mod files;
pub mod handler;
pub mod logging;
pub mod prompt;

/// Converts a Path/PathBuf into String type.
/// Uses unwrap(), which may panic for a path with
/// invalid utf-8 for certain OSes.
#[macro_export]
macro_rules! path_str {
    ($p:expr) => {
        $p.to_str().unwrap().to_string()
    };
}
