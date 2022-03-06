use crate::dotfile::{Dotfile, Handler};
use crate::files::{FileHandler, Sha256Digest, SystemFileHandler};
use crate::logging;
use crate::prompt::StdinPrompt;
use anyhow::{bail, Result};
use clap::{command, Arg, Command};
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as Cmd;

#[derive(Default)]
pub struct Cli;

impl Cli {
    pub fn exec(&self) -> Result<()> {
        let matches = command!()
            .about("Simple dotfile management")
            .subcommand_required(true)
            .arg(
                Arg::new("log")
                    .help("Display logs.")
                    .long("log")
                    .global(true)
                    .takes_value(true)
                    .possible_values(&["trace", "debug", "info", "warn", "error"])
                    .default_missing_value("info"),
            )
            .subcommand(
                Command::new("sync")
                    .about("Sync home and repo files, defaults FROM home TO repo.")
                    .arg(
                        Arg::new("home")
                            .long("home")
                            .help("Sync files from repository to home."),
                    )
                    .arg(Arg::new("dryrun").long("dryrun").short('d'))
                    .arg(
                        Arg::new("no-confirm")
                            .long("no-confirm")
                            .short('n')
                            .help("Skip prompt."),
                    )
                    .arg(
                        Arg::new("no-backup")
                            .long("no-backup")
                            .help("Do not create backups when copying to home."),
                    )
                    .arg(Arg::new("ignore-missing").long("ignore-missing").short('i')),
            )
            .subcommand(
                Command::new("status")
                    .about("Display the current status between home and repository.")
                    .arg(
                        Arg::new("brief")
                            .long("brief")
                            .short('b')
                            .help("Only display files that are not up to date."),
                    ),
            )
            .subcommand(
                Command::new("diff")
                    .about("Show diff between files that do not match.")
                    .arg(
                        Arg::new("command")
                            .long("command")
                            .short('c')
                            .help("Use as diff command (default: diff -u --color)")
                            .number_of_values(1),
                    ),
            )
            .subcommand(
                Command::new("edit").about("Edit ~/dotfiles.y[a]ml.").arg(
                    Arg::new("editor")
                        .long("editor")
                        .short('e')
                        .takes_value(true),
                ),
            )
            .subcommand(
                Command::new("git")
                    .about("Run arbitrary git command in repository.")
                    .long_about(
                        "Runs an arbitrary git command in the configured repository.\
Usage: dotfiles git -- <...>
Example: dotfiles git -- status",
                    )
                    .arg(Arg::new("args").multiple_values(true)),
            )
            .get_matches();

        if let Some(level) = matches.value_of("log") {
            logging::init(level)?
        }

        let home = get_home()?;
        log::debug!("Home directory: {}", home.to_str().unwrap());

        let dotfile_path = match get_dotfile_path(&home) {
            Some(path) => path,
            None => {
                println!("~/dotfiles.y[a]ml not found, creating new");
                let mut path = home.clone();
                path.push("dotfiles.yml");
                bootstrap(&path)?;
                return Ok(());
            }
        };

        let create_handler = || -> Result<Handler> {
            let digester = Sha256Digest::default();
            let file_handler = SystemFileHandler::default();

            let dotfile = load_dotfile(&dotfile_path, &file_handler)?;
            let repo = dotfile.repository();

            Ok(Handler::new(
                Box::new(file_handler),
                Box::new(digester),
                Box::new(StdinPrompt {}),
                home,
                repo,
                dotfile.files(),
            ))
        };

        match matches.subcommand() {
            Some(("edit", matches)) => {
                let editor = get_editor(matches.value_of("editor"));
                log::debug!("Editing using {}", editor);

                let mut cmd = Cmd::new(&editor);
                cmd.arg(&dotfile_path);
                cmd.status()?;
            }
            Some(("status", matches)) => {
                let handler = create_handler()?;

                let brief = matches.is_present("brief");
                handler.status(brief)?;
            }
            Some(("diff", matches)) => {
                let command = matches.value_of("command").map(|c| c.to_string());
                let handler = create_handler()?;
                handler.diff(command)?;
            }
            Some(("git", matches)) => {
                let file_handler = SystemFileHandler::default();
                let dotfile = load_dotfile(&dotfile_path, &file_handler)?;

                let mut cmd = Cmd::new("git");
                cmd.current_dir(dotfile.repository());

                match matches.values_of("args") {
                    Some(values) => {
                        for arg in values {
                            cmd.arg(arg);
                        }
                    }
                    None => todo!(),
                }
                cmd.status()?;
            }
            Some(("sync", matches)) => {
                let mut handler = create_handler()?;

                if matches.is_present("no-confirm") {
                    log::info!("Setting confirm=false");
                    handler.confirm(false);
                }

                if matches.is_present("no-backup") {
                    log::info!("Setting backup=false");
                    handler.backup(false);
                }

                if matches.is_present("dryrun") {
                    log::info!("Setting dryrun=true");
                    handler.dryrun(true);
                }

                if matches.is_present("ignore-missing") {
                    log::info!("Setting ignore-missing=true");
                    handler.ignore_invalid(true);
                }

                if matches.is_present("home") {
                    handler.confirm(true);
                    handler.copy_to_home()?;
                } else {
                    handler.copy_to_repo()?;
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

fn load_dotfile(path: &Path, file_handler: &SystemFileHandler) -> Result<Dotfile> {
    let s = file_handler.read_string(path)?;
    let dotfile = Dotfile::from(&s)?;
    Ok(dotfile)
}

fn get_dotfile_path(home: &Path) -> Option<PathBuf> {
    let mut a = home.to_path_buf();
    a.push("dotfiles.yml");

    let mut b = home.to_path_buf();
    b.push("dotfiles.yaml");

    if a.exists() {
        Some(a)
    } else if b.exists() {
        Some(b)
    } else {
        None
    }
}

fn get_home() -> Result<PathBuf> {
    match home::home_dir() {
        Some(home) => Ok(home),
        None => bail!("unable to resolve home directory"),
    }
}

fn bootstrap(path: &Path) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let current_dir = current_dir.to_str().unwrap();
    let s = format!("repository: {current_dir} files: []");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;
    file.write_all(s.as_bytes())?;
    Ok(())
}

fn get_editor(flag: Option<&str>) -> String {
    match flag {
        Some(s) => s.to_string(),
        None => match env::var("EDITOR") {
            Ok(s) => s,
            Err(_) => String::from("vim"),
        },
    }
}
