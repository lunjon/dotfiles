use crate::color;
use crate::files::{Digester, FileHandler};
use crate::prompt::Prompt;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[cfg(test)]
mod tests;

/// Dotfile represents the ~/dotfiles.yml file (called DF),
/// i.e the specification the user creates.
#[derive(Debug, Deserialize)]
pub struct Dotfile {
    // Path to the repository.
    repository: String,
    // Files that should be tracked.
    // Path can be either absolute or relative to ~/dotfiles.y[a]ml.
    files: Vec<String>,
}

impl Dotfile {
    pub fn from(s: &str) -> Result<Dotfile> {
        let df: Dotfile = serde_yaml::from_str(&s)?;

        // Validate that repository path exists
        let path = PathBuf::from(&df.repository);
        if !path.exists() {
            bail!("invalid repository path: {}", df.repository)
        }

        Ok(df)
    }

    pub fn repository(&self) -> PathBuf {
        PathBuf::from(&self.repository)
    }

    pub fn files(self) -> Vec<String> {
        self.files
    }
}

pub struct Handler {
    file_handler: Box<dyn FileHandler>,
    digester: Box<dyn Digester>,
    prompt: Box<dyn Prompt>,
    // The path to the users home directory.
    home: PathBuf,
    // The path to the repository to sync files to.
    repository: PathBuf,
    // The files read from the DF file.
    files: Vec<String>,
    // If a file is missing from the source (i.e where it is copied from),
    // ignore any error it is causing.
    ignore_invalid: bool,
    // Do not execute any file operations.
    dryrun: bool,
    // Ask user, by using the prompt field, to confirm each copy.
    confirm: bool,
    // When copying to home, create a backup file if it already exists.
    backup: bool,
}

impl Handler {
    pub fn new(
        file_handler: Box<dyn FileHandler>,
        digester: Box<dyn Digester>,
        prompt: Box<dyn Prompt>,
        home: PathBuf,
        repository: PathBuf,
        files: Vec<String>,
    ) -> Self {
        Self {
            backup: true,
            confirm: true,
            dryrun: false,
            ignore_invalid: false,
            digester,
            file_handler,
            files,
            home,
            prompt,
            repository,
        }
    }

    pub fn backup(&mut self, value: bool) {
        self.backup = value;
    }

    pub fn dryrun(&mut self, value: bool) {
        self.dryrun = value;
    }

    pub fn confirm(&mut self, value: bool) {
        self.confirm = value;
    }

    pub fn ignore_invalid(&mut self, value: bool) {
        self.ignore_invalid = value;
    }

    pub fn copy_to_home(&self) -> Result<()> {
        self.copy(Target::Home)
    }

    pub fn copy_to_repo(&self) -> Result<()> {
        self.copy(Target::Repo)
    }

    pub fn status(&self, brief: bool) -> Result<()> {
        log::debug!("Showing status with brief={}", brief);

        let entries = self.make_entries()?;
        let entries: Vec<&Entry> = entries.iter().filter(|e| !(brief && e.is_ok())).collect();

        for entry in entries {
            println!(" {}", entry);
        }

        if !brief {
            println!(
                "\n{} ok | {} diff | {} invalid | {} missing home | {} missing repository",
                Status::Ok,
                Status::Diff,
                Status::Invalid("".to_string()),
                Status::MissingHome,
                Status::MissingRepo,
            );
        }
        Ok(())
    }

    fn copy(&self, target: Target) -> Result<()> {
        let exec = !self.dryrun;
        let entries = self.make_entries()?;

        for entry in entries {
            match entry.status {
                Status::Ok => {
                    log::info!("{} ok", entry.name);
                    continue;
                }
                Status::Invalid(reason) => {
                    if self.ignore_invalid {
                        continue;
                    }
                    bail!("invalid entry: {}", reason);
                }
                Status::MissingHome if !target.is_home() => {
                    if self.ignore_invalid {
                        println!("Ignoring missing source: {}", entry.name);
                        continue;
                    }
                    bail!("missing source: {}", entry.name);
                }
                Status::MissingRepo if target.is_home() => {
                    if self.ignore_invalid {
                        println!("Ignoring missing source: {}", entry.name);
                        continue;
                    }
                    bail!("missing source: {}", entry.name);
                }
                _ => {}
            }

            let (src, dst) = match target {
                Target::Home => (entry.repo_path, entry.home_path),
                Target::Repo => (entry.home_path, entry.repo_path),
            };

            let src_str = src.to_str().unwrap();
            let dst_str = dst.to_str().unwrap();

            if self.confirm {
                let msg = format!(
                    "Copy {} to {}?",
                    color::green(src_str),
                    color::blue(dst_str)
                );
                let ok = self.prompt.confirm(&msg)?;

                if !ok {
                    log::info!("Skipping {}", src_str);
                    continue;
                }
            }

            let dir = match dst.parent() {
                Some(parent) => parent,
                None => bail!("failed to get parent directory of {}", dst_str),
            };

            if !dir.exists() && exec {
                let dir = dir.to_path_buf();
                log::info!("Creating directory: {}", dir.to_str().unwrap());
                self.file_handler.create_dirs(&dir)?;
            }

            if exec {
                if target.is_home() && dst.exists() && self.backup {
                    let backup = dst.with_extension("backup");
                    self.file_handler.copy(&dst, &backup)?;
                }
                log::debug!("Copy: {} to {}", src_str, dst_str);
                self.file_handler.copy(&src, &dst)?;
            }

            println!("  {} {}", color::green(""), &entry.name);
        }

        Ok(())
    }

    fn make_entries(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        for file in &self.files {
            log::debug!("Processing file: {}", file);
            let path = PathBuf::from(file);

            let mut home_path = self.home.clone();
            home_path.push(&path);
            let mut repo_path = self.repository.clone();
            repo_path.push(&path);

            if !path.is_relative() {
                log::debug!("Adding {} as invalid", file);
                let status = Status::Invalid(format!("path is not relative: {}", file));
                let entry = Entry::new(file, status, home_path, repo_path);
                entries.push(entry);
                continue;
            }

            if !(home_path.exists() || repo_path.exists()) {
                log::debug!("Adding {} as invalid", file);
                let entry = Entry::new(
                    file,
                    Status::Invalid("does not exists in either home or repository".to_string()),
                    home_path,
                    repo_path,
                );
                entries.push(entry);
                continue;
            }

            let mut make_entry = |h: PathBuf, r: PathBuf| -> Result<()> {
                if let Some(entry) = self.make_entry(h, r)? {
                    entries.push(entry);
                }
                Ok(())
            };

            let mut expand = |dir: &PathBuf| -> Result<()> {
                for file in Self::files_in(dir)? {
                    let mut h = home_path.clone();
                    h.push(&file);

                    let mut r = repo_path.clone();
                    r.push(&file);

                    make_entry(h, r)?;
                }
                Ok(())
            };

            if !home_path.exists() {
                if repo_path.is_dir() {
                    expand(&repo_path)?;
                } else {
                    make_entry(home_path, repo_path)?;
                }
            } else if !repo_path.exists() {
                if home_path.is_dir() {
                    expand(&home_path)?;
                } else {
                    make_entry(home_path, repo_path)?;
                }
            } else {
                // Both paths exist
                if home_path.is_dir() {
                    let mut files = Vec::new();
                    let mut files_at_home = Self::files_in(&home_path)?;
                    let mut files_at_repo = Self::files_in(&repo_path)?;

                    files.append(&mut files_at_home);
                    files.append(&mut files_at_repo);
                    files.sort();
                    files.dedup();

                    for file in files {
                        let mut h = home_path.clone();
                        h.push(&file);

                        let mut r = repo_path.clone();
                        r.push(&file);

                        make_entry(h, r)?;
                    }
                } else {
                    make_entry(home_path, repo_path)?;
                }
            }
        }

        Ok(entries)
    }

    fn make_entry(&self, home_path: PathBuf, repo_path: PathBuf) -> Result<Option<Entry>> {
        let name = match home_path.strip_prefix(&self.home) {
            Ok(p) => {
                let s = p.to_str().unwrap();
                if s.ends_with("backup") {
                    return Ok(None);
                }
                s.to_string()
            }
            Err(err) => {
                log::error!("failed to strip prefix: {}", err);
                return Ok(None);
            }
        };

        let status = self.get_status(&home_path, &repo_path)?;
        let entry = Entry {
            name,
            status,
            home_path,
            repo_path,
        };

        Ok(Some(entry))
    }

    fn get_status(&self, home_path: &Path, repo_path: &Path) -> Result<Status> {
        let status = if !home_path.exists() {
            Status::MissingHome
        } else if !repo_path.exists() {
            Status::MissingRepo
        } else {
            let s = self.file_handler.read_string(&home_path)?;
            let hash_src = self.digester.digest(s.as_bytes())?;

            let s = self.file_handler.read_string(&repo_path)?;
            let hash_dst = self.digester.digest(s.as_bytes())?;

            if hash_src.eq(&hash_dst) {
                Status::Ok
            } else {
                Status::Diff
            }
        };

        Ok(status)
    }

    fn files_in(dir: &Path) -> Result<Vec<String>> {
        if !dir.is_dir() {
            bail!("was not a directory");
        }
        log::debug!("Expanding directory: {}", dir.to_str().unwrap());

        let mut files = Vec::new();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() {
                let name = path.file_name().unwrap();
                let name = name.to_str().unwrap();
                files.push(name.to_string());
            }
        }
        Ok(files)
    }
}

struct Entry {
    name: String,
    status: Status,
    home_path: PathBuf,
    repo_path: PathBuf,
}

impl Entry {
    fn new(name: &str, status: Status, home_path: PathBuf, repo_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            status,
            home_path,
            repo_path,
        }
    }

    #[cfg(test)]
    fn is_invalid(&self) -> bool {
        matches!(self.status, Status::Invalid(_))
    }

    fn is_ok(&self) -> bool {
        matches!(self.status, Status::Ok)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.status, self.name)
    }
}

enum Status {
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

enum Target {
    Home,
    Repo,
}

impl Target {
    fn is_home(&self) -> bool {
        match self {
            Target::Home => true,
            Target::Repo => false,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Home => write!(f, "home"),
            Target::Repo => write!(f, "repo"),
        }
    }
}