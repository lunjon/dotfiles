use crate::data::Dotfile;
use crate::handler::{DiffHandler, DiffOptions, Only, StatusHandler, SyncHandler, SyncOptions};
use crate::logging;
use crate::prompt::StdinPrompt;
use crate::{files, CmdRunner};
use anyhow::{bail, Result};
use clap::{command, Arg, ArgMatches, Command};
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
                    .arg(Arg::new("dryrun").long("dryrun"))
                    .arg(
                        Arg::new("diff")
                            .long("diff")
                            .conflicts_with("no-confirm")
                            .help("Display inline diffs before"),
                    )
                    .arg(
                        Arg::new("diff-command")
                            .long("diff-command")
                            .help("Use as diff command (default: diff -u --color)")
                            .number_of_values(1),
                    )
                    .arg(
                        Arg::new("no-confirm")
                            .long("no-confirm")
                            .alias("yes")
                            .short('y')
                            .help("Skip confirmation prompt."),
                    )
                    .arg(
                        Arg::new("no-backup")
                            .long("no-backup")
                            .help("Do not create backups when copying to home."),
                    )
                    .arg(
                        Arg::new("only")
                            .help("Only include files matching patterns specified. Pattern uses glob by default. Set --regex to use regular expressions.")
                            .long("only")
                            .short('o')
                            .takes_value(true)
                            .multiple_occurrences(true)
                            .required(false),
                    )
                    .arg(
                        Arg::new("regex")
                            .help("Use regular expressions in patterns specified in --only.")
                            .long("regex")
                            .short('r')
                    )
                    .arg(Arg::new("ignore-missing").long("ignore-missing").short('i').help("Do not give error on missing files."))
                    .arg(
                        Arg::new("commit")
                            .help("Create a git commit after syncing files. Only valid when copying files to repository.")
                            .long("commit")
                            .short('C')
                            .takes_value(true)
                    )
                    .arg(
                        Arg::new("push")
                            .help("Run git push after commit.")
                            .long("push")
                            .takes_value(false)
                    ),
            )
            .subcommand(
                Command::new("status")
                    .alias("st")
                    .about("Display the current status between home and repository.")
                    .arg(
                        Arg::new("only")
                            .help("Only include files matching patterns specified. Pattern uses glob by default. Set --regex to use regular expressions.")
                            .long("only")
                            .short('o')
                            .takes_value(true)
                            .multiple_occurrences(true)
                            .required(false),
                    )
                    .arg(
                        Arg::new("regex")
                            .help("Use regular expressions in patterns specified in --only.")
                            .long("regex")
                            .short('r')
                    )
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
                        Arg::new("only")
                            .help("Only include files matching patterns specified. Pattern uses glob by default. Set --regex to use regular expressions.")
                            .long("only")
                            .short('o')
                            .takes_value(true)
                            .multiple_occurrences(true)
                            .required(false),
                    )
                    .arg(
                        Arg::new("regex")
                            .help("Use regular expressions in patterns specified in --only.")
                            .long("regex")
                            .short('r')
                    )
                    .arg(
                        Arg::new("diff-command")
                            .long("diff-command")
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
Usage: dotf git <...>
Example: dotf git status",
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
        log::debug!("Home directory: {:?}", home);

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

        match matches.subcommand() {
            None => {
                let dotfile = load_dotfile(&dotfile_path)?;
                let handler = StatusHandler::new(home, dotfile.repository(), dotfile.items(), None);
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
                let only = get_only(matches)?;
                let dotfile = load_dotfile(&dotfile_path)?;
                let handler = StatusHandler::new(home, dotfile.repository(), dotfile.items(), only);
                let brief = matches.is_present("brief");
                handler.status(brief)?;
            }
            Some(("diff", matches)) => {
                let only = get_only(matches)?;
                let dotfile = load_dotfile(&dotfile_path)?;
                let options = get_diff_options(matches)?;
                let handler =
                    DiffHandler::new(home, dotfile.repository(), dotfile.items(), options, only);
                handler.diff()?;
            }
            Some(("git", matches)) => {
                let dotfile = load_dotfile(&dotfile_path)?;
                let runner = CmdRunner::new(dotfile.repository());

                let args = match matches.values_of("args") {
                    Some(args) => args.map(|a| a.to_string()).collect::<Vec<String>>(),
                    None => vec![],
                };

                runner.run("git", args)?;
            }
            Some(("sync", matches)) => {
                let dotfile = load_dotfile(&dotfile_path)?;
                let only = get_only(matches)?;
                let diff_options = get_diff_options(matches)?;
                let options = SyncOptions {
                    confirm: !matches.is_present("no-confirm"),
                    backup: !matches.is_present("no-backup"),
                    dryrun: matches.is_present("dryrun"),
                    ignore_invalid: matches.is_present("ignore-missing"),
                    show_diff: matches.is_present("diff"),
                    diff_options,
                };

                let repository = dotfile.repository();
                let handler = SyncHandler::new(
                    Box::new(StdinPrompt {}),
                    home,
                    repository.clone(),
                    dotfile.items(),
                    options,
                    only,
                );

                if matches.is_present("home") {
                    handler.copy_to_home()?;
                } else {
                    handler.copy_to_repo()?;
                    if let Some(msg) = matches.value_of("commit") {
                        log::info!("Creating git commit with message: {msg}");
                        let runner = CmdRunner::new(repository);
                        runner.run("git", to_strings(&["add", "."]))?;
                        runner.run("git", to_strings(&["commit", "-m", msg]))?;

                        if matches.is_present("push") {
                            log::info!("Running git push");
                            runner.run("git", to_strings(&["push"]))?;
                        }
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

fn get_only(matches: &ArgMatches) -> Result<Option<Only>> {
    match matches.values_of("only") {
        Some(patterns) => {
            let patterns: Vec<&str> = patterns.into_iter().collect();
            log::debug!("Got --only: {:?}", &patterns);
            let o = match matches.is_present("regex") {
                true => Only::from_regex(&patterns)?,
                false => Only::from_glob(&patterns)?,
            };
            Ok(Some(o))
        }
        None => Ok(None),
    }
}

fn get_diff_options(matches: &ArgMatches) -> Result<DiffOptions> {
    match matches.value_of("diff-command") {
        Some(s) => {
            let split: Vec<String> = s.split_whitespace().map(|s| s.to_string()).collect();
            if split.is_empty() {
                bail!("empty diff command")
            }
            Ok(DiffOptions::new(split))
        }
        None => Ok(DiffOptions::default()),
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

fn to_strings(v: &[&str]) -> Vec<String> {
    v.to_vec().iter().map(|s| s.to_string()).collect()
}
