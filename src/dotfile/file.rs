use super::item::Item;
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_yaml::{from_str, from_value, Value};
use std::path::PathBuf;

/// Dotfile represents the ~/dotfiles.yml file (called DF),
/// i.e the specification the user creates.
#[derive(Debug)]
pub struct Dotfile {
    // Path to the repository.
    repository: String,
    // Files that should be tracked. Path must be relative to home.
    files: Vec<Item>,
}

impl Dotfile {
    pub fn from(s: &str) -> Result<Dotfile> {
        let df: InternalDotfile = from_str(s)?;

        // Validate that repository path exists
        let path = PathBuf::from(&df.repository);
        if !path.exists() {
            bail!("invalid repository path: {}", df.repository)
        }

        Ok(Dotfile {
            repository: df.repository.clone(),
            files: df.items()?,
        })
    }

    pub fn repository(&self) -> PathBuf {
        PathBuf::from(&self.repository)
    }

    pub fn items(self) -> Vec<Item> {
        self.files
    }
}

#[derive(Deserialize)]
struct InternalDotfile {
    repository: String,
    files: Vec<Value>,
}

impl InternalDotfile {
    fn items(self) -> Result<Vec<Item>> {
        let mut items = Vec::new();
        for file in self.files {
            let item = match file {
                Value::String(s) => Item::Filepath(s),
                Value::Mapping(m) => {
                    let obj: Obj = from_value(Value::Mapping(m))?;
                    Item::Object {
                        path: obj.path,
                        name: obj.name,
                    }
                }
                _ => bail!("invalid item at _"),
            };

            items.push(item);
        }

        Ok(items)
    }
}

#[derive(Deserialize)]
struct Obj {
    path: String,
    pub name: Option<String>,
}
