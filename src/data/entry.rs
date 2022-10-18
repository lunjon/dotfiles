use crate::color;
use crate::path_str;
use crate::HOME_DIR;
use anyhow::Result;
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
        let path = PathBuf::from(relpath);
        let relpath = match path.strip_prefix(HOME_DIR.as_str()) {
            Ok(p) => path_str!(p),
            Err(_) => relpath.to_string(),
        };

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
            Entry::Err(reason) => write!(f, "{} {}", color::red(""), reason),
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
            Status::Ok => color::green(""),
            Status::Diff => color::yellow(""),
            Status::MissingHome => color::yellow(""),
            Status::MissingRepo => color::yellow(""),
        };

        write!(f, "{}", icon)
    }
}
