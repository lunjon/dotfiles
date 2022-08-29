use super::indexer::Indexer;
use super::types::{DiffOptions, Only};
use crate::data::{Entry, Item};
use crate::path_str;
use anyhow::Result;
use std::path::PathBuf;

pub struct DiffHandler {
    indexer: Indexer,
    items: Vec<Item>,
    options: DiffOptions,
}

impl DiffHandler {
    pub fn new(
        home: PathBuf,
        repository: PathBuf,
        items: Vec<Item>,
        options: DiffOptions,
        only: Option<Only>,
    ) -> Self {
        let indexer = Indexer::new(home, repository, only);
        Self {
            indexer,
            items,
            options,
        }
    }
    pub fn diff(&self) -> Result<()> {
        let entries = self.indexer.index(&self.items)?;
        let entries: Vec<&Entry> = entries.iter().filter(|e| e.is_diff()).collect();

        if entries.is_empty() {
            println!("All up to date.");
            return Ok(());
        }

        for entry in entries {
            let a = path_str!(entry.home_path);
            let b = path_str!(entry.repo_path);

            let mut cmd = self.options.to_cmd(&a, &b)?;
            cmd.status()?;
        }
        Ok(())
    }
}
