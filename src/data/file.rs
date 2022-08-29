use super::item::Item;
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
    // Files that should be tracked. Path must be relative to home.
    files: Vec<Item>,
}

impl Dotfile {
    pub fn from(s: &str) -> Result<Dotfile> {
        let df: NewDotfile = toml::from_str(s)?;

        // Validate that repository path exists
        let path = PathBuf::from(&df.repository);
        if !path.exists() {
            bail!("invalid repository path: {}", df.repository)
        }

        let mut items = Vec::new();
        for (name, value) in df.files {
            match value {
                Toml::String(s) => {
                    if s.trim().is_empty() {
                        bail!("{}: string must not be empty", name);
                    }

                    let item = Item::Filepath(s);
                    items.push(item);
                }
                Toml::Array(arr) => {
                    if arr.is_empty() {
                        bail!("{}: list must not be empty", name);
                    }

                    for value in arr {
                        match value {
                            Toml::String(s) => items.push(Item::Filepath(s)),
                            _ => bail!("invalid type for {}", name),
                        }
                    }
                }
                Toml::Table(t) => {
                    let s = toml::to_string(&t)?;
                    let obj: Obj = toml::from_str(&s)?;
                    items.push(Item::Object {
                        ignore: obj.ignore,
                        path: obj.path,
                        name,
                    });
                }
                _ => bail!("invalid type for {}", name),
            }
        }

        Ok(Dotfile {
            repository: df.repository,
            files: items,
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
struct Obj {
    ignore: Option<Vec<String>>,
    path: String,
}

#[derive(Deserialize)]
struct NewDotfile {
    // Path to the repository.
    repository: String,
    files: HashMap<String, Toml>,
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
        object = { path = "text.txt" }
        with-ignore = { path = "text.txt", ignore = [ ".cache" ] }
        "#;

        let dotfile = Dotfile::from(dotfile_content).expect("valid dotfile");
        assert_eq!(dotfile.files.len(), 6);
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
