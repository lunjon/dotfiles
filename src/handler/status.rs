use super::types::Only;
use crate::data::Entry;
use crate::index::Indexer;
use crate::{
    color,
    data::{Item, Status},
};
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

        let mut indexed = self.indexer.index(&self.items)?;
        if brief {
            self.brief(indexed);
        } else {
            self.full(&mut indexed);
        }
        Ok(())
    }

    fn full(&self, indexed: &mut Vec<(String, Vec<Entry>)>) {
        indexed.sort_by(|(_, a), (_, b)| a.len().partial_cmp(&b.len()).unwrap());

        for (name, entries) in indexed {
            if entries.is_empty() {
                continue;
            }

            if entries.len() == 1 {
                println!(" {}", entries.first().unwrap());
            } else {
                println!("\n {}", name);
                for entry in entries {
                    println!("   {}", entry);
                }
            }
        }

        println!(
            "\n{} ok | {} diff | {} invalid | {} missing home | {} missing repository",
            Status::Ok,
            Status::Diff,
            color::red(""),
            Status::MissingHome,
            Status::MissingRepo,
        );
    }

    fn brief(&self, _entries: Vec<(String, Vec<Entry>)>) {

        // let entries: Vec<&Entry> = entries
        //     .iter()
        //     .flat_map(|(_name, es)| es)
        //     .filter(|e| !(brief && e.is_ok()))
        //     .collect();
    }
}
