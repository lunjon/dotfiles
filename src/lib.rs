use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;

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

struct CmdRunner {
    cwd: PathBuf,
}

impl CmdRunner {
    fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    fn run(&self, cmd: &str, args: Vec<String>) -> Result<()> {
        let mut cmd = Command::new(cmd);
        cmd.current_dir(&self.cwd);

        for arg in args {
            cmd.arg(arg);
        }
        cmd.status()?;
        Ok(())
    }
}
