use crate::cmd::CmdRunner;
use crate::data::Dotfile;
use crate::files;
use crate::handler::{DiffHandler, DiffOptions, Only, StatusHandler, SyncHandler, SyncOptions};
use crate::logging;
use crate::path::HOME_DIR;
use crate::prompt::StdinPrompt;
use anyhow::{bail, Result};
use clap::builder::PossibleValuesParser;
use clap::{command, Arg, ArgAction, ArgMatches, Command};
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
                    .value_parser(PossibleValuesParser::new(&["trace", "debug", "info", "warn", "error"]))
                    .default_missing_value("info"),
            )
            .subcommand(
                Command::new("sync")
                    .about("Sync home and repo files, defaults home -> repo.")
                    .arg(
                        Arg::new("home")
                            .help("Sync files from repository to home.")
                            .long("home")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(Arg::new("dryrun").long("dryrun"))
                    .arg(
                        Arg::new("diff")
                            .long("diff")
                            .action(ArgAction::SetTrue)
                            .conflicts_with("no-confirm")
                            .help("Display inline diffs before"),
                    )
                    .arg(
                        Arg::new("diff-command")
                            .help("Use as diff command (default: diff -u --color)")
                            .long("diff-command")
                            .requires("diff")
                            .number_of_values(1),
                    )
                    .arg(
                        Arg::new("no-confirm")
                            .help("Skip confirmation prompt.")
                            .long("no-confirm")
                            .alias("yes")
                            .short('y')
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("no-backup")
                            .help("Do not create backups when copying to home.")
                            .long("no-backup")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("interactive").help("Sync files interactively.")
                        .long("interactive")
                        .short('i')
                        .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("only")
                            .help("Only include files matching patterns specified. Pattern uses glob by default. Set --regex to use regular expressions.")
                            .long("only")
                            .short('o')
                            .takes_value(true)
                            .action(ArgAction::Append)
                            .required(false),
                    )
                    .arg(
                        Arg::new("regex")
                            .help("Use regular expressions in patterns specified in --only.")
                            .long("regex")
                            .short('r')
                            .requires("only")
                            .action(ArgAction::SetTrue)
                    )
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
                            .requires("commit")
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
                            .action(ArgAction::Append)
                            .required(false),
                    )
                    .arg(
                        Arg::new("regex")
                            .help("Use regular expressions in patterns specified in --only.")
                            .long("regex")
                            .short('r')
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("brief")
                            .long("brief")
                            .short('b')
                            .action(ArgAction::SetTrue)
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
                            .action(ArgAction::Append)
                            .required(false),
                    )
                    .arg(
                        Arg::new("regex")
                            .help("Use regular expressions in patterns specified in --only.")
                            .long("regex")
                            .short('r')
                            .action(ArgAction::SetTrue)
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

        if let Some(level) = matches.get_one::<String>("log") {
            logging::init(level)?
        }

        let home = PathBuf::from(HOME_DIR.as_str());
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
                let editor = get_editor(matches.get_one("editor"));
                log::debug!("Editing using {}", editor);

                let mut cmd = Cmd::new(&editor);
                cmd.arg(&dotfile_path);
                cmd.status()?;
            }
            Some(("status", matches)) => {
                let only = get_only(matches)?;
                let dotfile = load_dotfile(&dotfile_path)?;
                let handler = StatusHandler::new(home, dotfile.repository(), dotfile.items(), only);
                let brief = get_bool(matches, "brief");
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

                let args = match matches.get_many::<String>("args") {
                    Some(args) => args.map(String::from).collect::<Vec<String>>(),
                    None => vec![],
                };

                runner.run("git", args)?;
            }
            Some(("sync", matches)) => {
                let dotfile = load_dotfile(&dotfile_path)?;
                let only = get_only(matches)?;
                let diff_options = get_diff_options(matches)?;

                let options = SyncOptions {
                    interactive: get_bool(matches, "interactive"),
                    confirm: !get_bool(matches, "no-confirm"),
                    backup: !get_bool(matches, "no-backup"),
                    dryrun: get_bool(matches, "dryrun"),
                    show_diff: get_bool(matches, "diff"),
                    diff_options,
                    git_commit: matches.get_one::<String>("commit").map(String::from),
                    git_push: get_bool(matches, "push"),
                };

                let repository = dotfile.repository();
                let handler = SyncHandler::new(
                    Box::new(StdinPrompt {}),
                    home,
                    repository,
                    dotfile.items(),
                    options,
                    only,
                );

                if let Some(true) = matches.get_one::<bool>("home") {
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

fn get_only(matches: &ArgMatches) -> Result<Option<Only>> {
    match matches.get_many::<String>("only") {
        Some(patterns) => {
            let patterns: Vec<String> = patterns.map(|s| s.to_string()).collect();
            log::debug!("Got --only: {:?}", &patterns);
            let o = match get_bool(matches, "regex") {
                true => Only::from_regex(&patterns)?,
                false => Only::from_glob(&patterns)?,
            };
            Ok(Some(o))
        }
        None => Ok(None),
    }
}

fn get_diff_options(matches: &ArgMatches) -> Result<DiffOptions> {
    match matches.get_one::<&str>("diff-command") {
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

fn bootstrap(path: &Path) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let current_dir = current_dir.to_str().unwrap();
    let content = format!(
        r#"repository = "{}"

[home]
vim = ".vimrc"                 # type string
glob = "notes/**/*.txt"        # string with glob pattern
list = [ ".zshrc", ".bashrc" ] # list of strings
object = {{ files = ["scripts/*"], ignore = [ "*.out", ".cache" ] }}
"#,
        current_dir
    );

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn get_editor(flag: Option<&String>) -> String {
    match flag {
        Some(s) => s.to_string(),
        None => match env::var("EDITOR") {
            Ok(s) => s,
            Err(_) => String::from("vim"),
        },
    }
}

fn get_bool(matches: &ArgMatches, name: &str) -> bool {
    matches
        .get_one::<bool>(name)
        .map_or(false, |v| v.to_owned())
}
