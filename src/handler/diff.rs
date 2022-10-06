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
        let entries: Vec<&Entry> = entries
            .iter()
            .flat_map(|(_name, es)| es)
            .filter(|e| e.is_diff())
            .collect();

        if entries.is_empty() {
            println!("All up to date.");
            return Ok(());
        }

        for entry in entries {
            if let Entry::Ok {
                home_path,
                repo_path,
                ..
            } = entry
            {
                let a = path_str!(home_path);
                let b = path_str!(repo_path);

                let mut cmd = self.options.to_cmd(&a, &b)?;
                cmd.status()?;
            }
        }
        Ok(())
    }
}
