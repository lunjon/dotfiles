pub mod cli;
pub mod color;
pub mod dotfile;
pub mod files;
pub mod logging;
pub mod prompt;

#[macro_export]
macro_rules! path_str {
    ($p:expr) => {
        $p.to_str().unwrap().to_string()
    };
}
