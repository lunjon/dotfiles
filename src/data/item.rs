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
        Self {
            name,
            files: vec![file],
            ignore: None,
        }
    }

    pub fn from_list(name: String, files: Vec<String>) -> Self {
        Self {
            name,
            files,
            ignore: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.files.iter().any(|path| match path.as_str() {
            "" | "*" | "**" | "**/*" => false,
            s => !(s.starts_with("**") || s.starts_with('/')),
        })
    }

    // pub fn get_path(&self) -> &str {
    //     match self {
    //         Item::Single(s) => s.trim(),
    //         Item::Object { path, .. } => path.trim(),
    //     }
    // }

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

// #[cfg(test)]
// mod tests {
//     use super::Item;

//     #[test]
//     fn test_valid_path() {
//         let tests = vec![
//             (false, Single("".to_string())),
//             (false, Single("/root".to_string())),
//             (false, Single("*".to_string())),
//             (false, Single("**".to_string())),
//             (false, Single("**/*".to_string())),
//             (true, Single(".tmux.conf".to_string())),
//             (true, Single(".config/test.yml".to_string())),
//         ];

//         for (valid, item) in tests {
//             assert_eq!(valid, item.is_valid());
//         }
//     }

//     #[test]
//     fn test_is_glob() {
//         let tests = vec![
//             (true, Single("*".to_string())),
//             (true, Single("**".to_string())),
//             (true, Single("**/*".to_string())),
//             (true, Single(".config/*".to_string())),
//             (false, Single(".tmux.conf".to_string())),
//             (false, Single(".config/test.yml".to_string())),
//             (false, Single(".config/".to_string())),
//         ];

//         for (valid, item) in tests {
//             assert_eq!(valid, item.is_glob());
//         }
//     }
// }
