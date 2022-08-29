use crate::prompt::Prompt;
use anyhow::Result;

#[cfg(test)]
mod integration;
#[cfg(test)]
mod unit;

pub struct PromptMock;

impl Prompt for PromptMock {
    fn prompt(&self, _msg: &str) -> Result<String> {
        Ok("yes".to_string())
    }
}
