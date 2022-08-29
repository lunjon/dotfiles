use crate::color;
use crate::data::{Item, Status};
use crate::files;
use crate::path_str;
use crate::prompt::Prompt;
use anyhow::{bail, Result};
use std::fmt;
use std::path::PathBuf;

use super::indexer::Indexer;
use super::types::{DiffOptions, Only};

pub struct SyncOptions {
    // If a file is missing from the source (i.e where it is copied from),
    // ignore any error it is causing.
    pub ignore_invalid: bool,
    // Do not execute any file operations.
    pub dryrun: bool,
    // Ask user, by using the prompt field, to confirm each copy.
    pub confirm: bool,
    // When copying to home, create a backup file if it already exists.
    pub backup: bool,
    pub show_diff: bool,
    pub diff_options: DiffOptions,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            ignore_invalid: false,
            dryrun: false,
            confirm: true,
            backup: true,
            show_diff: false,
            diff_options: DiffOptions::default(),
        }
    }
}

pub struct SyncHandler {
    indexer: Indexer,
    prompt: Box<dyn Prompt>,
    items: Vec<Item>,
    options: SyncOptions,
}

// Public methods.
impl SyncHandler {
    pub fn new(
        prompt: Box<dyn Prompt>,
        home: PathBuf,
        repository: PathBuf,
        items: Vec<Item>,
        options: SyncOptions,
        only: Option<Only>,
    ) -> Self {
        let indexer = Indexer::new(home, repository, only);
        Self {
            options,
            prompt,
            indexer,
            items,
        }
    }

    pub fn copy_to_home(&self) -> Result<()> {
        self.copy(Target::Home)
    }

    pub fn copy_to_repo(&self) -> Result<()> {
        self.copy(Target::Repo)
    }

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
                let prefix = if self.options.show_diff {
                    let mut cmd = self.options.diff_options.to_cmd(&src_str, &dst_str)?;
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
