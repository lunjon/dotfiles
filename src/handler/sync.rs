use super::indexer::Indexer;
use super::types::{DiffOptions, Only};
use crate::cmd::CmdRunner;
use crate::color;
use crate::data::{Entry, Item, Status};
use crate::files;
use crate::path_str;
use crate::prompt::Prompt;
use anyhow::{bail, Result};
use std::fmt;
use std::path::PathBuf;

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
    // Creates a git commit message.
    pub git_commit: Option<String>,
    // Run git push after committing.
    pub git_push: bool,
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
            git_commit: None,
            git_push: false,
        }
    }
}

pub struct SyncHandler {
    indexer: Indexer,
    prompt: Box<dyn Prompt>,
    items: Vec<Item>,
    options: SyncOptions,
    runner: CmdRunner,
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
        let runner = CmdRunner::new(repository.clone());
        let indexer = Indexer::new(home, repository, only);
        Self {
            options,
            prompt,
            indexer,
            items,
            runner,
        }
    }

    pub fn copy_to_home(&self) -> Result<()> {
        self.copy(Target::Home)
    }

    pub fn copy_to_repo(&self) -> Result<()> {
        self.copy(Target::Repo)?;
        if let Some(msg) = &self.options.git_commit {
            log::info!("Creating git commit with message: {msg}");
            self.runner.run("git", to_strings(&["add", "."]))?;
            self.runner
                .run("git", to_strings(&["commit", "-m", msg.as_str()]))?;

            if self.options.git_push {
                log::info!("Running git push");
                self.runner.run("git", to_strings(&["push"]))?;
            }
        }
        Ok(())
    }

    fn copy(&self, target: Target) -> Result<()> {
        let exec = !self.options.dryrun;
        let map = self.indexer.index(&self.items)?;
        let entries: Vec<&Entry> = map.iter().flat_map(|(_name, es)| es).collect();

        for entry in entries {
            match entry {
                Entry::Ok {
                    relpath,
                    status,
                    home_path,
                    repo_path,
                } => {
                    match status {
                        Status::Ok => {
                            log::info!("{} ok", relpath);
                            continue;
                        }
                        Status::MissingHome if !target.is_home() => continue,
                        Status::MissingRepo if target.is_home() => continue,
                        _ => {}
                    }

                    let (display_name, src, dst) = match target {
                        Target::Home => {
                            let s = format!("~/{}", relpath);
                            (s, repo_path, home_path)
                        }
                        Target::Repo => {
                            let s = path_str!(repo_path);
                            (s.to_string(), home_path, repo_path)
                        }
                    };

                    let src_str = path_str!(src);
                    let dst_str = path_str!(dst);

                    if self.options.confirm {
                        let prefix = if self.options.show_diff && matches!(status, Status::Diff) {
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

                            files::copy(dst, &backup)?;
                            log::debug!("Created backup of {}", dst_str);
                        }

                        files::copy(src, dst)?;
                    }

                    println!("  {} {}", color::green(""), &relpath);
                }
                Entry::Err(reason) => bail!("invalid entry: {}", reason),
            }
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

fn to_strings(v: &[&str]) -> Vec<String> {
    v.to_vec().iter().map(|s| s.to_string()).collect()
}
