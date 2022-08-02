#[derive(Debug)]
pub enum Item {
    Filepath(String),
    Object { path: String, name: Option<String> },
}

impl Item {
    pub fn valid_path(&self) -> bool {
        let path = self.get_path();
        match path {
            "" | "*" | "**" | "**/*" => false,
            s => !(s.starts_with("**") || s.starts_with('/')),
        }
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
            assert_eq!(valid, item.valid_path());
        }
    }
}
