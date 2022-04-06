use anyhow::Result;
use std::io::{self, Write};

pub trait Prompt {
    fn prompt(&self, msg: &str) -> Result<String>;

    fn confirm(&self, msg: &str, default_yes: bool) -> Result<bool> {
        let query = if default_yes { "[Y/n]" } else { "[y/N]" };

        let msg = format!("{msg}? {query} ");
        let answer = self.prompt(&msg)?;

        let answer = match answer.to_lowercase().trim() {
            "" => default_yes,
            "y" | "yes" => true,
            "n" | "no" => false,
            _ => false,
        };

        Ok(answer)
    }
}

pub struct StdinPrompt {}

impl Prompt for StdinPrompt {
    fn prompt(&self, msg: &str) -> Result<String> {
        let mut stdout = io::stdout();
        write!(stdout, "{msg}")?;
        stdout.flush()?;

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        Ok(buf.trim().to_string())
    }
}
