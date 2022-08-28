use crate::color;
use crate::files;
use crate::path_str;
use crate::prompt::Prompt;
use anyhow::{bail, Result};
use glob::Pattern as GlobPattern;
use regex::Regex;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

mod entry;
mod file;
mod indexer;
mod item;
#[cfg(test)]
mod tests;

pub use entry::{Entry, Status};
pub use file::Dotfile;
pub use item::Item;

use self::indexer::Indexer;

// TODO: remove after refactored into multiple handlers
#[derive(Clone)]
enum Pattern {
    Glob(GlobPattern),
    Regex(Regex),
}

impl Pattern {
    fn matches(&self, s: &str) -> bool {
        match self {
            Pattern::Glob(g) => g.matches(s),
            Pattern::Regex(g) => g.is_match(s),
        }
    }
}

// TODO: remove after refactored into multiple handlers
#[derive(Clone)]
pub struct Only {
    patterns: Vec<Pattern>,
}

impl Only {
    pub fn from_glob(patterns: &[&str]) -> Result<Self> {
        let mut ps = Vec::new();
        for p in patterns {
            let g = GlobPattern::new(p)?;
            ps.push(Pattern::Glob(g));
        }
        Ok(Self { patterns: ps })
    }

    pub fn from_regex(patterns: &[&str]) -> Result<Self> {
        let mut ps = Vec::new();
        for p in patterns {
            let r = Regex::new(p)?;
            ps.push(Pattern::Regex(r));
        }
        Ok(Self { patterns: ps })
    }
}

pub struct Options {
    // If a file is missing from the source (i.e where it is copied from),
    // ignore any error it is causing.
    pub ignore_invalid: bool,
    // Do not execute any file operations.
    pub dryrun: bool,
    // Ask user, by using the prompt field, to confirm each copy.
    pub confirm: bool,
    // When copying to home, create a backup file if it already exists.
    pub backup: bool,
    pub sync_show_diff: bool,
    // Only include files matching these patterns.
    pub only: Option<Only>,
    pub diff_command: Option<Vec<String>>,
}

impl Options {
    fn get_diff_command(&self) -> Result<Vec<String>> {
        match &self.diff_command {
            None => Ok(vec![
                String::from("diff"),
                String::from("-u"),
                String::from("--color"),
            ]),
            Some(v) => Ok(v.to_vec()),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            ignore_invalid: false,
            dryrun: false,
            confirm: true,
            backup: true,
            sync_show_diff: false,
            only: None,
            diff_command: None,
        }
    }
}

pub struct Handler {
    indexer: Indexer,
    prompt: Box<dyn Prompt>,
    items: Vec<Item>,
    options: Options,
}

// Public methods.
impl Handler {
    pub fn new(
        prompt: Box<dyn Prompt>,
        home: PathBuf,
        repository: PathBuf,
        items: Vec<Item>,
        options: Options,
    ) -> Self {
        let indexer = Indexer::new(home, repository, options.only.clone());
        Self {
            options,
            prompt,
            indexer,
            items,
        }
    }

    pub fn with_options(&mut self, options: Options) {
        self.options = options;
    }

    pub fn copy_to_home(&self) -> Result<()> {
        self.copy(Target::Home)
    }

    pub fn copy_to_repo(&self) -> Result<()> {
        self.copy(Target::Repo)
    }

    pub fn diff(&self) -> Result<()> {
        let entries = self.indexer.index(&self.items)?;
        let entries: Vec<&Entry> = entries.iter().filter(|e| e.is_diff()).collect();

        if entries.is_empty() {
            println!("All up to date.");
            return Ok(());
        }

        for entry in entries {
            let a = path_str!(entry.home_path);
            let b = path_str!(entry.repo_path);

            let mut cmd = self.make_diff_command(&a, &b)?;
            cmd.status()?;
        }
        Ok(())
    }

    pub fn status(&self, brief: bool) -> Result<()> {
        log::debug!("Showing status with brief={}", brief);

        let entries = self.indexer.index(&self.items)?;
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
        let exec = !self.options.dryrun;
        let entries = self.indexer.index(&self.items)?;

        for entry in entries {
            match entry.status {
                Status::Ok => {
                    log::info!("{} ok", entry.relpath);
                    continue;
                }
                Status::Invalid(reason) => {
                    if self.options.ignore_invalid {
                        continue;
                    }
                    bail!("invalid entry: {}", reason);
                }
                Status::MissingHome if !target.is_home() => continue,
                Status::MissingRepo if target.is_home() => continue,
                _ => {}
            }

            let (display_name, src, dst) = match target {
                Target::Home => {
                    let s = format!("~/{}", entry.relpath);
                    (s, entry.repo_path, entry.home_path)
                }
                Target::Repo => {
                    let s = path_str!(entry.repo_path);
                    (s.to_string(), entry.home_path, entry.repo_path)
                }
            };

            let src_str = path_str!(src);
            let dst_str = path_str!(dst);

            if self.options.confirm {
                let prefix = if self.options.sync_show_diff {
                    let mut cmd = self.make_diff_command(&src_str, &dst_str)?;
                    cmd.status()?;
                    "\n  "
                } else {
                    ""
                };

                let msg = format!("{}Write {}?", prefix, color::blue(&display_name));
                if !self.prompt.confirm(&msg, false)? {
                    log::info!("Skipping {}", src_str);
                    continue;
                }
            }

            let dir = match dst.parent() {
                Some(parent) => parent,
                None => bail!("failed to get parent directory of {}", dst_str),
            };

            if !dir.exists() && exec {
                log::info!("Creating directory: {:?}", dir);
                files::create_dirs(dir)?;
            }

            if exec {
                if target.is_home() && dst.exists() && self.options.backup {
                    let filename = path_str!(dst.file_name().unwrap());
                    let filename = format!("{filename}.backup");

                    let mut backup = PathBuf::from(&dst);
                    backup.set_file_name(filename);

                    files::copy(&dst, &backup)?;
                    log::debug!("Created backup of {}", dst_str);
                }

                files::copy(&src, &dst)?;
            }

            println!("  {} {}", color::green(""), &entry.relpath);
        }

        Ok(())
    }
}

impl Handler {
    pub fn make_diff_command(&self, a: &str, b: &str) -> Result<Command> {
        let (root, args) = {
            let mut cmd = self.options.get_diff_command()?;
            let first = cmd.remove(0);
            (first, cmd)
        };

        let mut cmd = Command::new(&root);
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.arg(a);
        cmd.arg(b);

        Ok(cmd)
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
