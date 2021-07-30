use crate::dotfile::{Dotfile, Handler};
use crate::files::{FileHandler, Sha256Digest, SystemFileHandler};
use crate::logging;
use crate::prompt::StdinPrompt;
use anyhow::{bail, Result};
use clap::{App, AppSettings, Arg};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Cli;

impl Cli {
    pub fn exec(&self) -> Result<()> {
        let matches = App::new("dotfiles")
            .about("Simple dotfile management")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .arg(
                Arg::new("log")
                    .about("Display logs.")
                    .long("log")
                    .global(true)
                    .takes_value(true)
                    .possible_values(&["trace", "debug", "info", "warn", "error"])
                    .default_missing_value("info"),
            )
            .subcommand(
                App::new("sync")
                    .about("Sync home <-> repo files.")
                    .arg(
                        Arg::new("home")
                            .long("home")
                            .about("Sync files from repository to home."),
                    )
                    .arg(Arg::new("dryrun").long("dryrun").short('d'))
                    .arg(
                        Arg::new("no-confirm")
                            .long("no-confirm")
                            .short('n')
                            .about("Skip prompt."),
                    )
                    .arg(
                        Arg::new("no-backup")
                            .long("no-backup")
                            .short('n')
                            .about("Do not create backups when copying to home."),
                    )
                    .arg(Arg::new("ignore-missing").long("ignore-missing").short('i')),
            )
            .subcommand(
                App::new("status")
                    .about("Display the current status between home and repository.")
                    .arg(
                        Arg::new("brief")
                            .long("brief")
                            .short('b')
                            .about("Only display files that are not up to date."),
                    ),
            )
            .subcommand(
                App::new("edit").about("Edit ~/dotfiles.y[a]ml.").arg(
                    Arg::new("editor")
                        .long("editor")
                        .short('e')
                        .takes_value(true)
                        .env("EDITOR"),
                ),
            )
            .get_matches();

        if let Some(level) = matches.value_of("log") {
            logging::init(level)?
        }

        let home = get_home()?;
        log::debug!("Home directory: {}", home.to_str().unwrap());

        let dotfile = match get_dotfile_path(&home) {
            Some(path) => path,
            None => {
                println!("~/dotfiles.y[a]ml not found, creating new");
                let mut path = home.clone();
                path.push("dotfiles.yml");
                bootstrap(&path)?;
                return Ok(());
            }
        };

        let subcmd = matches.subcommand_name().unwrap();
        let matches = matches.subcommand_matches(subcmd).unwrap();

        match subcmd {
            "edit" => {
                let editor = matches.value_of("editor").unwrap_or("vim");
                log::debug!("Editing using {}", editor);

                let mut cmd = std::process::Command::new(&editor);
                cmd.arg(&dotfile);
                cmd.status()?;
            }
            "status" => {
                let digester = Sha256Digest::default();
                let file_handler = SystemFileHandler::default();

                let dotfile = load_dotfile(&dotfile, &file_handler)?;
                let repo = dotfile.repository();

                let handler = Handler::new(
                    Box::new(file_handler),
                    Box::new(digester),
                    Box::new(StdinPrompt {}),
                    home,
                    repo,
                    dotfile.files(),
                );

                let brief = matches.is_present("brief");
                handler.status(brief)?;
            }
            "sync" => {
                let digester = Sha256Digest::default();
                let file_handler = SystemFileHandler::default();

                let dotfile = load_dotfile(&dotfile, &file_handler)?;
                let repo = dotfile.repository();

                let mut handler = Handler::new(
                    Box::new(file_handler),
                    Box::new(digester),
                    Box::new(StdinPrompt {}),
                    home,
                    repo,
                    dotfile.files(),
                );

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
    let s = file_handler.read_string(&path)?;
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
    let s = format!(
        "repository: {}
files: []",
        current_dir
    );

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;
    file.write_all(s.as_bytes())?;
    Ok(())
}
