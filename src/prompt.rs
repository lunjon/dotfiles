use anyhow::{bail, Result};
use std::io::{self, Write};

pub trait Prompt {
    fn prompt(&self, msg: &str) -> Result<String>;

    fn confirm(&self, msg: &str) -> Result<bool> {
        let msg = format!("{} [y/N] ", msg);
        let answer = self.prompt(&msg)?;
        let lower = answer.to_lowercase();

        let answer = match lower.as_str() {
            "y" | "yes" => true,
            "n" | "no" => false,
            e => bail!("invalid answer: {}", e),
        };

        Ok(answer)
    }
}

pub struct StdinPrompt {}

impl Prompt for StdinPrompt {
    fn prompt(&self, msg: &str) -> Result<String> {
        let mut buf = String::new();
        print!("{}", msg);
        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;
        Ok(buf.trim().to_string())
    }
}
