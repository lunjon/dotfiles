#[derive(Debug)]
pub enum Item {
    Filepath(String),
    Object { path: String, name: Option<String> },
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

    fn get_path(&self) -> &str {
        match self {
            Item::Filepath(s) => s.trim(),
            Item::Object { path, .. } => path.trim(),
        }
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
