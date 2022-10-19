use super::types::Only;
use crate::data::Entry;
use crate::data::{Item, Status};
use crate::index::Indexer;
use anyhow::Result;
use crossterm::style::Stylize;
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

        let mut indexed = self.indexer.index(&self.items)?;
        indexed.sort_by(|(_, a), (_, b)| a.len().partial_cmp(&b.len()).unwrap());

        if brief {
            let mut filtered = Vec::new();
            for (name, entries) in indexed {
                let entries: Vec<Entry> = entries
                    .into_iter()
                    .filter(|entry| !entry.is_status_ok())
                    .collect();
                if !entries.is_empty() {
                    filtered.push((name, entries));
                }
            }
            self.display(&filtered);
        } else {
            self.display(&indexed);
            println!(
                "\n{} ok | {} diff | {} invalid | {} missing home | {} missing repository",
                Status::Ok,
                Status::Diff,
                "".red(),
                Status::MissingHome,
                Status::MissingRepo,
            );
        }
        Ok(())
    }

    fn display(&self, indexed: &[(String, Vec<Entry>)]) {
        for (name, entries) in indexed {
            if entries.is_empty() {
                continue;
            }

            if entries.len() == 1 {
                println!(" {}: {}", name, entries.first().unwrap());
            } else {
                println!("\n {}", name);
                for entry in entries {
                    println!("   {}", entry);
                }
            }
        }
    }
}
