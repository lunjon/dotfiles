use super::item::Item;
use crate::path::{try_strip_home_prefix, LOCAL_CONFIG_DIR};
use anyhow::{bail, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use toml::Value as Toml;

/// Dotfile represents the ~/dotfiles.yml file (called DF),
/// i.e the specification the user creates.
#[derive(Debug)]
pub struct Dotfile {
    // Path to the repository.
    repository: String,
    // Files that should be tracked.
    items: Vec<Item>,
}

impl Dotfile {
    pub fn from(s: &str) -> Result<Dotfile> {
        let df: RawDotfile = toml::from_str(s)?;

        // Validate that repository path exists
        let path = PathBuf::from(&df.repository);
        if !path.exists() {
            bail!("invalid repository path: {}", df.repository)
        }

        let mut items = Vec::new();
        if let Some(map) = df.files {
            for (name, value) in map {
                let item = Item::from_toml(name, value)?;
                items.push(item);
            }
        }

        if let Some(map) = df.config {
            let relative = try_strip_home_prefix(&LOCAL_CONFIG_DIR);
            for (name, value) in map {
                let item = Item::from_toml(name, value)?.with_suffix(&relative);
                items.push(item);
            }
        }

        Ok(Dotfile {
            repository: df.repository,
            items,
        })
    }

    pub fn repository(&self) -> PathBuf {
        PathBuf::from(&self.repository)
    }

    pub fn items(self) -> Vec<Item> {
        self.items
    }
}

type ItemMap = HashMap<String, Toml>;

// The type which is read from file.
#[derive(Deserialize)]
struct RawDotfile {
    // Path to the repository.
    repository: String,
    files: Option<ItemMap>,
    config: Option<ItemMap>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let dotfile_content = r#"
        repository = "./"

        [files]
        cargo = "Cargo.toml"
        docs = [ "README.md", "todo.norg" ]
        glob = "src/*.rs"
        object = { files = ["text.txt" ]}
        with-ignore = { files = ["text.txt"], ignore = [ ".cache" ] }
        "#;

        let dotfile = Dotfile::from(dotfile_content).expect("valid dotfile");
        assert_eq!(dotfile.items.len(), 5);
    }

    #[test]
    fn test_from_invalid() {
        let tests = [
            "empty = \"\"",
            "other-empty = []",
            "empty-object = {}",
            "boolean = true",
            "integer = 1",
        ];
        for item in tests {
            let dotfile_content = format!(
                r#"
        repository = "./"
        [files]
        {}
        "#,
                item
            );
            let res = Dotfile::from(&dotfile_content);
            assert!(res.is_err());
        }
    }
}
