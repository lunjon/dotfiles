use anyhow::Result;
use glob::Pattern;

#[derive(Clone, Debug)]
pub enum Item {
    Filepath(String),
    Object {
        path: String,
        name: String,
        ignore: Option<Vec<String>>,
    },
}

impl Item {
    pub fn is_valid(&self) -> bool {
        let path = self.get_path();
        match path {
            "" | "*" | "**" | "**/*" => false,
            s => !(s.starts_with("**") || s.starts_with('/')),
        }
    }

    pub fn is_glob(&self) -> bool {
        let path = self.get_path();
        path.contains('*')
    }

    pub fn get_path(&self) -> &str {
        match self {
            Item::Filepath(s) => s.trim(),
            Item::Object { path, .. } => path.trim(),
        }
    }

    pub fn ignore_patterns(&self) -> Result<Option<Vec<Pattern>>> {
        let ps = match self {
            Item::Filepath(_) => None,
            Item::Object { ignore, .. } => match ignore {
                None => None,
                Some(v) => {
                    let mut ps = Vec::new();
                    for s in v {
                        let pattern = Pattern::new(s)?;
                        ps.push(pattern);
                    }
                    Some(ps)
                }
            },
        };

        Ok(ps)
    }
}

#[cfg(test)]
mod tests {
    use super::Item::*;

    #[test]
    fn test_valid_path() {
        let tests = vec![
            (false, Filepath("".to_string())),
            (false, Filepath("/root".to_string())),
            (false, Filepath("*".to_string())),
            (false, Filepath("**".to_string())),
            (false, Filepath("**/*".to_string())),
            (true, Filepath(".tmux.conf".to_string())),
            (true, Filepath(".config/test.yml".to_string())),
        ];

        for (valid, item) in tests {
            assert_eq!(valid, item.is_valid());
        }
    }

    #[test]
    fn test_is_glob() {
        let tests = vec![
            (true, Filepath("*".to_string())),
            (true, Filepath("**".to_string())),
            (true, Filepath("**/*".to_string())),
            (true, Filepath(".config/*".to_string())),
            (false, Filepath(".tmux.conf".to_string())),
            (false, Filepath(".config/test.yml".to_string())),
            (false, Filepath(".config/".to_string())),
        ];

        for (valid, item) in tests {
            assert_eq!(valid, item.is_glob());
        }
    }
}
