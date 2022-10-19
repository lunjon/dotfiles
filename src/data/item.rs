use std::path::PathBuf;

use anyhow::{bail, Result};
use glob::Pattern;
use serde::Deserialize;
use toml::{self, Value as Toml};

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub files: Vec<PathBuf>,
    pub ignore: Option<Vec<String>>,
}

#[macro_export]
macro_rules! item {
    ($name:expr, $files:expr) => {{
        let files = {
            let v: Vec<String> = $files.to_vec().iter().map(|s| s.to_string()).collect();
            v
        };
        $crate::data::item::Item::new($name.to_string(), files, None)
    }};
    ($name:expr, $files:expr, $ignore:expr) => {{
        let files = {
            let v: Vec<String> = $files.to_vec().iter().map(|s| s.to_string()).collect();
            v
        };
        let ignore = {
            let v: Vec<String> = $ignore.to_vec().iter().map(|s| s.to_string()).collect();
            v
        };
        $crate::data::item::Item::new($name.to_string(), files, Some(ignore))
    }};
}

impl Item {
    pub fn new(name: String, files: Vec<String>, ignore: Option<Vec<String>>) -> Self {
        Self {
            name,
            ignore,
            files: files.iter().map(PathBuf::from).collect(),
        }
    }

    pub fn from_str(name: String, file: String) -> Self {
        Self::new(name, vec![file], None)
    }

    pub fn from_list(name: String, files: Vec<String>) -> Self {
        Self::new(name, files, None)
    }

    pub fn from_toml(name: String, value: Toml) -> Result<Self> {
        let item = match value {
            Toml::String(s) => {
                if s.trim().is_empty() {
                    bail!("{}: string must not be empty", name);
                }
                Self::from_str(name, s)
            }
            Toml::Array(arr) => {
                if arr.is_empty() {
                    bail!("{}: list must not be empty", name);
                }

                let mut files = Vec::new();
                for value in arr {
                    match value {
                        Toml::String(s) => files.push(s),
                        _ => bail!("invalid type for {}", name),
                    }
                }
                Self::from_list(name, files)
            }
            Toml::Table(t) => {
                let s = toml::to_string(&t)?;
                let obj: Obj = toml::from_str(&s)?;
                Self::new(name, obj.files, obj.ignore)
            }
            _ => bail!("invalid type for {}", name),
        };
        Ok(item)
    }

    pub fn with_suffix(mut self, suffix: &str) -> Self {
        let root = PathBuf::from(suffix);
        self.files = self.files.iter().map(|p| root.join(p)).collect();
        self
    }

    pub fn ignore_patterns(&self) -> Result<Option<Vec<Pattern>>> {
        let patterns = match &self.ignore {
            None => None,
            Some(v) => {
                let mut ps = Vec::new();
                for s in v {
                    ps.push(Pattern::new(s)?);
                }
                Some(ps)
            }
        };
        Ok(patterns)
    }
}

#[derive(Deserialize)]
struct Obj {
    ignore: Option<Vec<String>>,
    files: Vec<String>,
}

#[cfg(test)]
impl Item {
    pub fn simple_new(name: &str, file: &str) -> Item {
        Item::from_str(name.to_string(), file.to_string())
    }

    pub fn object_new(name: &str, files: &[&str], ignore: Option<&[&str]>) -> Item {
        let files = files
            .to_vec()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let ignore = ignore.map(|f| {
            f.to_vec()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        });
        Item::new(name.to_string(), files, ignore)
    }
}
