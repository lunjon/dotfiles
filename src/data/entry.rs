use crate::path::try_strip_home_prefix;
use anyhow::Result;
use crossterm::style::Stylize;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone)]
pub enum Entry {
    Ok {
        // The relative filepath for the dotfile, e.g .gitconfig
        relpath: String,
        status: Status,
        home_path: PathBuf,
        repo_path: PathBuf,
    },
    Err(String),
}

impl Entry {
    pub fn new(
        relpath: &str,
        status: Status,
        home_path: PathBuf,
        repo_path: PathBuf,
    ) -> Result<Self> {
        let relpath = try_strip_home_prefix(relpath);
        Ok(Self::Ok {
            relpath,
            status,
            home_path,
            repo_path,
        })
    }

    pub fn new_err(reason: String) -> Self {
        Self::Err(reason)
    }

    pub fn is_ok(&self) -> bool {
        match self {
            Entry::Ok { .. } => true,
            Entry::Err(_) => false,
        }
    }

    pub fn is_status_ok(&self) -> bool {
        match self {
            Entry::Ok { status, .. } => matches!(status, Status::Ok),
            Entry::Err(_) => false,
        }
    }

    pub fn is_diff(&self) -> bool {
        match self {
            Entry::Ok { status, .. } => matches!(status, Status::Diff),
            Entry::Err(_) => false,
        }
    }

    pub fn get_relpath(&self) -> &str {
        if let Self::Ok { relpath, .. } = &self {
            relpath
        } else {
            panic!()
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::Ok {
                relpath, status, ..
            } => write!(f, "{} {}", status, relpath),
            Entry::Err(reason) => write!(f, "{} {}", "".red(), reason),
        }
    }
}

#[derive(Clone)]
pub enum Status {
    Ok,
    Diff,
    MissingHome,
    MissingRepo,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let icon = match self {
            Status::Ok => "".green(),
            Status::Diff => "".yellow(),
            Status::MissingHome => "".yellow(),
            Status::MissingRepo => "".yellow(),
        };

        write!(f, "{}", icon)
    }
}
