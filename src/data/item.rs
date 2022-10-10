use anyhow::Result;
use glob::Pattern;

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub files: Vec<String>,
    pub ignore: Option<Vec<String>>,
}

impl Item {
    pub fn new(name: String, files: Vec<String>, ignore: Option<Vec<String>>) -> Self {
        Self {
            name,
            files,
            ignore,
        }
    }

    pub fn from_str(name: String, file: String) -> Self {
        Self::new(name, vec![file], None)
    }

    pub fn from_list(name: String, files: Vec<String>) -> Self {
        Self::new(name, files, None)
    }

    pub fn is_valid(&self) -> bool {
        self.files.iter().any(|path| match path.as_str() {
            "" | "*" | "**" | "**/*" => false,
            s => !(s.starts_with("**") || s.starts_with('/')),
        })
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
