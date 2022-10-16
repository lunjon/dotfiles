use directories::BaseDirs;
use lazy_static::lazy_static;

pub mod cli;
pub mod cmd;
pub mod color;
pub mod data;
pub mod files;
pub mod handler;
pub mod index;
pub mod logging;
pub mod prompt;

#[cfg(test)]
mod testing;

/// Converts a Path/PathBuf into String type.
/// Uses unwrap(), which may panic for a path with /// invalid utf-8 for certain OSes.
#[macro_export]
macro_rules! path_str {
    ($p:expr) => {
        $p.to_str().expect("valid utf-8 string path").to_string()
    };
}

lazy_static! {
    pub static ref SEP: String = std::path::MAIN_SEPARATOR.to_string();
    static ref BASE_DIRS: BaseDirs = BaseDirs::new().expect("to resolve base directory");
    pub static ref HOME_DIR: String = path_str!(BASE_DIRS.home_dir());
    pub static ref LOCAL_CONFIG_DIR: String = path_str!(BASE_DIRS.config_dir());
    pub static ref LOCAL_DATA_DIR: String = path_str!(BASE_DIRS.data_local_dir());
}

/// Builds a configuration path based on CONFIG_DIR.
#[macro_export]
macro_rules! config_path {
    ($($p:expr),*) => {
        {
            let mut v = Vec::new();
            v.push(crate::LOCAL_CONFIG_DIR.as_str());
            $(
                v.push($p);
            )*
            v.join(&crate::SEP)
        }
   }
}

/// Builds a path based on local data directory.
#[macro_export]
macro_rules! data_path {
    ($($p:expr),*) => {
        {
            let mut v = Vec::new();
            v.push(crate::LOCAL_DATA_DIR.as_str());
            $(
                v.push($p);
            )*
            v.join(&crate::SEP)
        }
   }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let path = config_path!("nvim", "lua", "init.lua");
        assert!(path.contains("nvim/lua/init"));
    }

    #[test]
    fn test_data_path() {
        let path = data_path!("nvim", "lua", "init.lua");
        assert!(path.contains("nvim/lua/init"));
    }
}
