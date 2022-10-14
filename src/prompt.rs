use anyhow::Result;
use inquire::{Confirm, Text};

pub trait Prompt {
    fn prompt(&self, msg: &str) -> Result<String>;
    fn confirm(&self, msg: &str, default_yes: bool) -> Result<bool>;
}

pub struct StdinPrompt {}

impl Prompt for StdinPrompt {
    fn prompt(&self, msg: &str) -> Result<String> {
        let text = Text::new(msg).prompt()?;
        Ok(text)
    }

    fn confirm(&self, msg: &str, default_yes: bool) -> Result<bool> {
        let ok = Confirm::new(msg).with_default(default_yes).prompt()?;
        Ok(ok)
    }
}
