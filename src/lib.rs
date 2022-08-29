pub mod cli;
pub mod color;
pub mod data;
pub mod files;
pub mod handler;
pub mod logging;
pub mod prompt;

#[macro_export]
macro_rules! path_str {
    ($p:expr) => {
        $p.to_str().unwrap().to_string()
    };
}
