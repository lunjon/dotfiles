use crate::color;
use std::fmt;
use std::path::PathBuf;

pub struct Entry {
    pub name: String,
    pub status: Status,
    pub home_path: PathBuf,
    pub repo_path: PathBuf,
}

impl Entry {
    pub fn new(name: &str, status: Status, home_path: PathBuf, repo_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            status,
            home_path,
            repo_path,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    fn is_invalid(&self) -> bool {
        matches!(self.status, Status::Invalid(_))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self.status, Status::Ok)
    }

    pub fn is_diff(&self) -> bool {
        matches!(self.status, Status::Diff)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.status, self.name)
    }
}

pub enum Status {
    Ok,
    Diff,
    MissingHome,
    MissingRepo,
    Invalid(String),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let icon = match self {
            Status::Ok => color::green(""),
            Status::Diff => color::yellow(""),
            Status::Invalid(_) => color::red(""),
            Status::MissingHome => color::yellow(""),
            Status::MissingRepo => color::yellow(""),
        };

        write!(f, "{}", icon)
    }
}
