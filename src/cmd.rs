use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

#[allow(dead_code)]
pub struct Output {
    stdout: String,
    stderr: String,
}

/// Used to run external commands, such as git.
pub struct CmdRunner {
    cwd: PathBuf,
}

impl CmdRunner {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    pub fn run(&self, cmd: &str, args: Vec<String>) -> Result<()> {
        let mut cmd = self.build(cmd, args);
        cmd.status()?;
        Ok(())
    }

    pub fn capture(&self, cmd: &str, args: Vec<String>) -> Result<Output> {
        let mut cmd = self.build(cmd, args);
        let output = cmd.output()?;
        let stdout = from_utf8(&output.stdout)?;
        let stderr = from_utf8(&output.stderr)?;
        Ok(Output {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        })
    }

    fn build(&self, cmd: &str, args: Vec<String>) -> Command {
        log::info!("Running {} with args '{}'", cmd, args.join(" "));

        let mut cmd = Command::new(cmd);
        cmd.current_dir(&self.cwd);

        for arg in args {
            cmd.arg(arg);
        }
        cmd
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::CmdRunner;

    fn setup() -> CmdRunner {
        CmdRunner::new(PathBuf::from("."))
    }

    #[test]
    fn test_run() {
        let runner = setup();
        let res = runner.capture("cargo", vec!["--help".to_string()]).unwrap();
        assert!(!res.stdout.is_empty());
        assert!(res.stderr.is_empty());
    }
}
