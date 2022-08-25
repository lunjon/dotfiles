use crate::dotfile::{Dotfile, Handler, Options};
use crate::files;
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
                            .short('y')
                            .help("Skip confirmation prompt."),
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
                    .alias("st")
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
                    .trailing_var_arg(true)
                    .about("Run arbitrary git command in repository.")
                    .long_about(
                        "Runs an arbitrary git command in the configured repository.\
Usage: dotf git -- <...>
Example: dotf git -- status",
                    )
                    .arg(
                        Arg::new("args")
                            .takes_value(true)
                            .multiple_values(true)
                            .allow_hyphen_values(true),
                    ),
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
                println!("~/.config/dotfiles.toml not found, creating new");
                let mut path = home.clone();
                path.push(".config");
                path.push("dotfiles.toml");
                bootstrap(&path)?;
                return Ok(());
            }
        };

        let create_handler = || -> Result<Handler> {
            let dotfile = load_dotfile(&dotfile_path)?;
            let repo = dotfile.repository();

            Ok(Handler::new(
                Box::new(StdinPrompt {}),
                home,
                repo,
                dotfile.items(),
            ))
        };

        match matches.subcommand() {
            None => {
                let handler = create_handler()?;
                handler.status(false)?;
            }
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
                let dotfile = load_dotfile(&dotfile_path)?;
                let mut cmd = Cmd::new("git");
                cmd.current_dir(dotfile.repository());

                if let Some(values) = matches.values_of("args") {
                    for arg in values {
                        cmd.arg(arg);
                    }
                }
                cmd.status()?;
            }
            Some(("sync", matches)) => {
                let options = Options {
                    confirm: !matches.is_present("no-confirm"),
                    backup: !matches.is_present("no-backup"),
                    dryrun: matches.is_present("dryrun"),
                    ignore_invalid: matches.is_present("ignore-missing"),
                };

                let mut handler = create_handler()?;
                handler.with_options(options);

                if matches.is_present("home") {
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

fn load_dotfile(path: &Path) -> Result<Dotfile> {
    let s = files::read_string(path)?;
    let dotfile = Dotfile::from(&s)?;
    Ok(dotfile)
}

fn get_dotfile_path(home: &Path) -> Option<PathBuf> {
    let mut path = home.to_path_buf();
    path.push(".config");
    path.push("dotfiles.toml");

    if path.exists() {
        Some(path)
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
    let content = format!(
        r#"repository = "{}"

[files]
vim = ".vimrc" # type string
glob = "notes/**/*.txt" # string with glob pattern
list = [ ".zshrc", ".bashrc" ] # list of strings
object = {{ path = "scripts/*", ignore = [ "*.out", ".cache" ] }}"#,
        current_dir
    );

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;
    file.write_all(content.as_bytes())?;
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
