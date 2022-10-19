use crate::data::{Entry, Item, Status};
use crate::handler::Only;
use crate::path::try_strip_prefix;
use crate::{files, path_str};
use anyhow::Result;
use glob::Pattern as GlobPattern;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub struct Indexer {
    // The path to the users home directory.
    home: PathBuf,
    home_str: String,
    // The path to the repository to sync files to.
    repo: PathBuf,
    repo_str: String,
    ignore_patterns: Vec<GlobPattern>,
    only: Option<Only>,
}

impl Indexer {
    pub fn new(home: PathBuf, repo: PathBuf, only: Option<Only>) -> Self {
        let home_str = path_str!(home);
        let repo_str = path_str!(repo);

        Self {
            home,
            home_str,
            repo,
            repo_str,
            ignore_patterns: vec![
                GlobPattern::new("*/.git/*").unwrap(),
                GlobPattern::new("*/node_modules/*").unwrap(),
                GlobPattern::new("*/target/*").unwrap(),
                GlobPattern::new("*.o").unwrap(),
                GlobPattern::new("*.backup").unwrap(),
                // Python
                GlobPattern::new("/*__pycache__/*").unwrap(),
                GlobPattern::new("*/.venv/*").unwrap(),
            ],
            only,
        }
    }

    pub fn index(&self, items: &[Item]) -> Result<Vec<(String, Vec<Entry>)>> {
        let mut entries: Vec<(String, Vec<Entry>)> = Vec::new();

        for item in items {
            let t = self.process_item(item)?;

            // TODO: refactor
            let mut filtered = Vec::new();
            if let Some(only) = &self.only {
                for entry in t {
                    if let Entry::Ok { relpath, .. } = &entry {
                        for pattern in &only.patterns {
                            if pattern.matches(relpath) {
                                filtered.push(entry);
                                break;
                            }
                        }
                    }
                }
            } else {
                filtered = t;
            }

            entries.push((item.name.clone(), filtered));
        }

        entries.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
        Ok(entries)
    }

    fn process_item(&self, item: &Item) -> Result<Vec<Entry>> {
        let ps = match item.ignore_patterns()? {
            None => vec![], // unnecessary allocation?
            Some(ps) => ps,
        };

        let mut entries = Vec::new();
        for path in &item.files {
            let filepath = path_str!(path);
            let home_path = self.home.join(&filepath);
            let repo_path = self.repo.join(&filepath);

            if is_glob(&filepath) {
                return self.process_glob(&filepath, &ps);
            }

            if !(home_path.exists() || repo_path.exists()) {
                let entry =
                    Entry::new_err("does not exists in either home or repository".to_string());
                return Ok(vec![entry]);
            }

            if home_path.is_dir() || repo_path.is_dir() {
                let fixed = match filepath.strip_suffix('/') {
                    Some(s) => format!("{}/*", s),
                    None => format!("{}/*", filepath),
                };

                let entry = Entry::new_err(format!(
                    "use glob pattern (fix: change {} to {})",
                    filepath, fixed,
                ));
                return Ok(vec![entry]);
            }

            if let Some(entry) = self.make_entry(&filepath, home_path, repo_path)? {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn process_glob(
        &self,
        globpattern: &str,
        ignore_patterns: &[GlobPattern],
    ) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();

        let home_glob_path = self.home.join(&globpattern);
        let repo_glob_path = self.repo.join(&globpattern);

        let home_str = path_str!(home_glob_path);
        let repo_str = path_str!(repo_glob_path);

        let home_glob = glob::glob(&home_str);
        let repo_glob = glob::glob(&repo_str);

        if home_glob.is_err() || repo_glob.is_err() {
            let entry = Entry::new_err(format!("invalid glob pattern: {}", globpattern));
            return Ok(vec![entry]);
        }

        let mut home_files: Vec<String> = Vec::new();
        for p in home_glob.unwrap().flatten() {
            if p.is_file() {
                let s = try_strip_prefix(&self.home_str, &path_str!(&p));
                if should_ignore(&s, &self.ignore_patterns) {
                    continue;
                }

                if !ignore_patterns.is_empty() && should_ignore(&s, ignore_patterns) {
                    continue;
                }

                log::debug!("Adding home file: {s}");
                home_files.push(s.to_string());
            }
        }

        let mut repo_files: Vec<String> = Vec::new();
        for p in repo_glob.unwrap().flatten() {
            if p.is_file() {
                let s = try_strip_prefix(&self.repo_str, &path_str!(&p));
                if should_ignore(&s, &self.ignore_patterns) {
                    continue;
                }

                if !ignore_patterns.is_empty() && should_ignore(&s, ignore_patterns) {
                    continue;
                }

                log::debug!("Adding repo file: {s}");
                repo_files.push(s.to_string());
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
            let h = self.home.join(path);
            let r = self.repo.join(path);

            match status {
                Some(status) => {
                    let entry = Entry::new(path, status, h, r)?;
                    entries.push(entry);
                }
                None => {
                    if let Some(entry) = self.make_entry(path, h, r)? {
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
            add_entry(s, Some(Status::MissingHome))?;
        }

        Ok(entries)
    }

    fn make_entry(
        &self,
        filepath: &str,
        home_path: PathBuf,
        repo_path: PathBuf,
    ) -> Result<Option<Entry>> {
        if home_path.ends_with("backup") {
            return Ok(None);
        }

        let status = get_status(&home_path, &repo_path)?;
        let entry = Entry::new(filepath, status, home_path, repo_path)?;
        Ok(Some(entry))
    }
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

fn should_ignore(path: &str, patterns: &[GlobPattern]) -> bool {
    patterns.iter().any(|p| p.matches(path))
}

pub fn is_glob(s: &str) -> bool {
    s.contains('*')
}
