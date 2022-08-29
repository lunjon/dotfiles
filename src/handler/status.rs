use super::indexer::Indexer;
use super::types::Only;
use crate::data::{Entry, Item, Status};
use anyhow::Result;
use std::path::PathBuf;

pub struct StatusHandler {
    indexer: Indexer,
    items: Vec<Item>,
}

// Public methods.
impl StatusHandler {
    pub fn new(home: PathBuf, repository: PathBuf, items: Vec<Item>, only: Option<Only>) -> Self {
        let indexer = Indexer::new(home, repository, only);
        Self { indexer, items }
    }

    pub fn status(&self, brief: bool) -> Result<()> {
        log::debug!("Showing status with brief={}", brief);

        let entries = self.indexer.index(&self.items)?;
        let entries: Vec<&Entry> = entries.iter().filter(|e| !(brief && e.is_ok())).collect();

        for entry in entries {
            println!(" {}", entry);
        }

        if !brief {
            println!(
                "\n{} ok | {} diff | {} invalid | {} missing home | {} missing repository",
                Status::Ok,
                Status::Diff,
                Status::Invalid("".to_string()),
                Status::MissingHome,
                Status::MissingRepo,
            );
        }
        Ok(())
    }
}
