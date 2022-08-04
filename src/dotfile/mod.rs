use crate::color;
use crate::files;
use crate::prompt::Prompt;
use anyhow::{bail, Result};
use glob::Pattern;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::spawn;

mod entry;
mod file;
mod item;
#[cfg(test)]
mod tests;

pub use entry::{Entry, Status};
pub use file::Dotfile;
pub use item::Item;

// TODO:
//  - refactor options (boolean values) into a struct
//  - speed up make_entries using threads (then use rayon crate)
pub struct Handler {
    prompt: Box<dyn Prompt>,
    // The path to the users home directory.
    home_str: String,
    // The path to the repository to sync files to.
    repository_str: String,
    // The files read from the DF file.
    items: Vec<Item>,
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

// Public methods.
impl Handler {
    pub fn new(
        prompt: Box<dyn Prompt>,
        home: PathBuf,
        repository: PathBuf,
        files: Vec<Item>,
    ) -> Self {
        let home_str = home.to_str().expect("valid home directory").to_string();
        let repository_str = repository
            .to_str()
            .expect("valid repository directory")
            .to_string();

        let _ignore_patterns = vec![
            glob::Pattern::new("**/.git/**/*").unwrap(),
            glob::Pattern::new("**/node_modules/**/*").unwrap(),
            glob::Pattern::new("**/target/**/*").unwrap(),
            glob::Pattern::new("*.o").unwrap(),
        ];

        Self {
            backup: true,
            confirm: true,
            dryrun: false,
            ignore_invalid: false,
            items: files,
            home_str,
            prompt,
            repository_str,
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

    pub fn diff(&self, cmd: Option<String>) -> Result<()> {
        let entries = self.make_entries()?;
        let entries: Vec<&Entry> = entries.iter().filter(|e| e.is_diff()).collect();

        if entries.is_empty() {
            println!("All up to date.");
            return Ok(());
        }

        let (root, args) = match cmd {
            Some(cstr) => {
                log::debug!("Using custom diff command: {}", cstr);

                let mut split: Vec<String> =
                    cstr.split_whitespace().map(|s| s.to_string()).collect();
                if split.is_empty() {
                    bail!("empty diff command")
                }

                let first = split.remove(0);
                (first, split)
            }
            None => (
                "diff".to_string(),
                vec!["-u".to_string(), "--color".to_string()],
            ),
        };

        for entry in entries {
            let a = entry.home_path.to_str().expect("to get home path");
            let b = entry.repo_path.to_str().expect("to get repo path");

            let mut cmd = Command::new(&root);
            for arg in &args {
                cmd.arg(arg);
            }
            cmd.arg(a);
            cmd.arg(b);

            cmd.status()?;
        }
        Ok(())
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
}

// Private methods.
impl Handler {
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
                Status::MissingHome if !target.is_home() => continue,
                Status::MissingRepo if target.is_home() => continue,
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
                let ok = self.prompt.confirm(&msg, false)?;

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
                files::create_dirs(&dir)?;
            }

            if exec {
                if target.is_home() && dst.exists() && self.backup {
                    let filename = dst.file_name().unwrap().to_str().unwrap();
                    let filename = format!("{filename}.backup");

                    let mut backup = PathBuf::from(&dst);
                    backup.set_file_name(filename);

                    files::copy(&dst, &backup)?;

                    log::debug!("Created backup of {}", dst_str);
                }
                log::debug!("Copy: {} to {}", src_str, dst_str);
                files::copy(&src, &dst)?;
            }

            println!("  {} {}", color::green(""), &entry.name);
        }

        Ok(())
    }

    fn make_entries(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        let mut threads = Vec::new();

        let home = self.home_str.to_string();
        let repo = self.repository_str.to_string();
        let items: Vec<Vec<Item>> = self.items.clone().chunks(8).map(|ch| ch.to_vec()).collect();

        for item in items {
            let i = item.clone();
            let h = home.clone();
            let r = repo.clone();
            let t = spawn(move || worker(h, r, i));
            threads.push(t);
        }

        for t in threads {
            let rs = t.join().unwrap()?;
            entries.extend(rs);
        }

        Ok(entries)
    }
}

fn worker(home: String, repo: String, items: Vec<Item>) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let patterns = vec![
        glob::Pattern::new("**/.git/**/*").unwrap(),
        glob::Pattern::new("**/node_modules/**/*").unwrap(),
        glob::Pattern::new("**/target/**/*").unwrap(),
        glob::Pattern::new("*.o").unwrap(),
    ];

    for item in items {
        let t = process_item(&home, &repo, &item, &patterns)?;
        entries.extend(t)
    }
    Ok(entries)
}

fn make_entry(home_path: PathBuf, repo_path: PathBuf) -> Result<Option<Entry>> {
    let name = match home_path.file_name() {
        Some(p) => {
            let s = p.to_str().unwrap();
            if s.ends_with("backup") {
                return Ok(None);
            }
            s.to_string()
        }
        None => return Ok(None),
    };

    let status = get_status(&home_path, &repo_path)?;
    let entry = Entry {
        name,
        status,
        home_path,
        repo_path,
    };

    Ok(Some(entry))
}

fn get_status(home_path: &Path, repo_path: &Path) -> Result<Status> {
    let status = if !home_path.exists() {
        Status::MissingHome
    } else if !repo_path.exists() {
        Status::MissingRepo
    } else {
        let s = files::read_string(home_path)?;
        let hash_src = files::digest(s.as_bytes())?;

        let s = files::read_string(repo_path)?;
        let hash_dst = files::digest(s.as_bytes())?;

        if hash_src.eq(&hash_dst) {
            Status::Ok
        } else {
            Status::Diff
        }
    };

    Ok(status)
}

fn ignore(path: &str, patterns: &[Pattern]) -> bool {
    patterns.iter().any(|p| p.matches(path))
}

fn process_glob(home: &str, repo: &str, item: &Item, patterns: &[Pattern]) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();

    let filepath = item.get_path();
    let path = PathBuf::from(filepath);

    let home_path = PathBuf::from(&home);
    let repo_path = PathBuf::from(&repo);

    let mut home_glob_path = PathBuf::from(&home);
    home_glob_path.push(&path);

    let mut repo_glob_path = PathBuf::from(&repo);
    repo_glob_path.push(&path);

    let home_str = home_glob_path.to_str().unwrap();
    let repo_str = repo_glob_path.to_str().unwrap();

    let home_glob = glob::glob(home_str);
    let repo_glob = glob::glob(repo_str);

    if home_glob.is_err() || repo_glob.is_err() {
        log::warn!("Error expanding home and repo glob pattern");
        let status = Status::Invalid("invalid glob pattern".to_string());
        let entry = Entry::new(filepath, status, home_glob_path, repo_glob_path);
        return Ok(vec![entry]);
    }

    let mut home_files: Vec<String> = Vec::new();
    for p in home_glob.unwrap().flatten() {
        if p.is_file() {
            let rel = p.strip_prefix(&home)?;
            let s = rel.to_str().unwrap();
            if !ignore(s, &patterns) {
                home_files.push(s.to_string());
            }
        }
    }

    let mut repo_files: Vec<String> = Vec::new();
    for p in repo_glob.unwrap().flatten() {
        if p.is_file() {
            let rel = p.strip_prefix(&repo)?;
            let s = rel.to_str().unwrap();
            if !ignore(s, &patterns) {
                repo_files.push(s.to_string());
            }
        }
    }

    let both: Vec<&String> = home_files
        .iter()
        .filter(|s| repo_files.contains(s))
        .collect();

    let home_only: Vec<&String> = home_files
        .iter()
        .filter(|s| !repo_files.contains(s))
        .collect();

    let repo_only: Vec<&String> = repo_files
        .iter()
        .filter(|s| !home_files.contains(s))
        .collect();

    let mut add_entry = |path: &str, status: Option<Status>| -> Result<()> {
        let h = home_path.join(path);
        let r = repo_path.join(path);
        match status {
            Some(status) => {
                let entry = Entry::new(path, status, h, r);
                entries.push(entry);
            }
            None => {
                if let Some(entry) = make_entry(h, r)? {
                    entries.push(entry);
                }
            }
        }
        Ok(())
    };

    for s in both {
        add_entry(s, None)?;
    }

    for s in home_only {
        add_entry(s, Some(Status::MissingRepo))?;
    }

    for s in repo_only {
        add_entry(s, Some(Status::MissingRepo))?;
    }

    Ok(entries)
}

fn process_item(home: &str, repo: &str, item: &Item, patterns: &[Pattern]) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();

    let filepath = match &item {
        Item::Filepath(s) => s.to_string(),
        Item::Object { path, .. } => path.to_string(),
    };

    let path = PathBuf::from(&filepath);
    let mut home_path = PathBuf::from(&home);
    home_path.push(&filepath);
    let mut repo_path = PathBuf::from(&repo);
    repo_path.push(&filepath);

    if !item.is_valid() {
        let status = Status::Invalid("path is invalid".to_string());
        let entry = Entry::new(&filepath, status, home_path, repo_path);
        return Ok(vec![entry]);
    }

    if item.is_glob() {
        return process_glob(home, repo, item, patterns);
    }

    if !path.is_relative() {
        let status = Status::Invalid(format!("path is not relative: {filepath}"));
        let entry = Entry::new(&filepath, status, home_path, repo_path);
        return Ok(vec![entry]);
    }

    if !(home_path.exists() || repo_path.exists()) {
        let entry = Entry::new(
            &filepath,
            Status::Invalid("does not exists in either home or repository".to_string()),
            home_path,
            repo_path,
        );
        return Ok(vec![entry]);
    }

    if home_path.is_dir() || repo_path.is_dir() {
        let fixed = match filepath.strip_suffix('/') {
            Some(s) => format!("{}/*", s),
            None => format!("{}/*", filepath),
        };

        let entry = Entry::new(
            &filepath,
            Status::Invalid(format!(
                "use glob pattern (fix: change {} to {})",
                filepath, fixed,
            )),
            home_path,
            repo_path,
        );
        return Ok(vec![entry]);
    }

    if let Some(entry) = make_entry(home_path, repo_path)? {
        entries.push(entry);
    }
    Ok(entries)
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
