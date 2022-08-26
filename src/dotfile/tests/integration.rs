use super::unit::*;
use crate::dotfile::{Handler, Item::*, Options, Status};
use crate::prompt::Prompt;
use anyhow::Result;
use std::path::PathBuf;

macro_rules! item {
    ($path:expr) => {
        Filepath(String::from($path))
    };
    ($path:expr, $($x:expr),*) => {
        {
            let mut tmp = Vec::new();
            $(
                tmp.push(String::from($x));
            )*

            Object {
                path: String::from($path),
                name: String::from("FIXME"),
                ignore: Some(tmp),
            }
        }
    };
}

pub struct PromptMock;

impl Prompt for PromptMock {
    fn prompt(&self, _msg: &str) -> Result<String> {
        Ok("yes".to_string())
    }
}

struct Fixture {
    context: TestContext,
    handler: Handler,
}

impl Fixture {
    /// Setup for integration tests by creating actual files.
    /// The structure looks like:
    ///   tmp-*/
    ///     home/
    ///       ignored.txt       <- ignored
    ///       diffed.txt        <- has diff
    ///       diffed.txt.backup <- should be ignored
    ///       init.vim          <- no diff
    ///       tmux.conf         <- new file
    ///       config/
    ///         spaceship.yml   <- new file
    ///         .git            <- should be ignored
    ///       deepglob/
    ///         config.yml      <- not in repo
    ///         node_modules    <- should be ignored
    ///         .git            <- should be ignored
    ///         test.out        <- should be ignored
    ///         src/
    ///           file.js       <- has diff
    ///     repo/
    ///       diffed.txt        <- has diff
    ///       init.vim          <- no diff
    ///       env.toml          <- not in home
    ///       deepglob/
    ///         src/
    ///            file.js      <- has diff
    ///
    /// The tmp-* directory is removed after each test.
    fn setup() -> Self {
        let context = TestContext::new(vec![
            FileSpec::target("ignored.txt", Status::MissingRepo),
            FileSpec::target("diffed.txt", Status::Diff),
            FileSpec::target("init.vim", Status::Ok),
            FileSpec::target("tmux.conf", Status::MissingRepo),
            FileSpec::target("config/spaceship.yml", Status::MissingRepo),
            FileSpec::target("env.toml", Status::MissingHome),
            FileSpec::target("deepglob/config.yml", Status::MissingRepo),
            FileSpec::target("deepglob/src/file.js", Status::Diff),
            FileSpec::special("deepglob/test.out"),
            FileSpec::special("deepglob/.git/config"),
            FileSpec::special("deepglob/.git/objects/abc123"),
            FileSpec::special("deepglob/.git/objects/def456"),
        ]);

        context.setup().unwrap();

        let items = vec![
            item!("diffed.txt"),
            item!("tmux.conf"),
            item!("init.vim"),
            item!("env.toml"),
            item!("config/*"),
            item!("deepglob/**/*", "*.out"),
        ];

        let options = Options {
            ignore_invalid: true,
            dryrun: false,
            confirm: false,
            backup: true,
            only: None,
        };
        let handler = Handler::new(
            Box::new(PromptMock {}),
            context.home_dir.clone(),
            context.repo_dir.clone(),
            items,
            options,
        );

        Self { context, handler }
    }

    fn home_path(&self, path: &str) -> PathBuf {
        self.context.home_path(path)
    }

    fn repo_path(&self, path: &str) -> PathBuf {
        self.context.repo_path(path)
    }
}

#[test]
fn copy_to_repo() {
    // Arrange
    let fixture = Fixture::setup();
    let tmuxconf = fixture.repo_path("tmux.conf");
    assert!(!tmuxconf.exists());
    let spaceship = fixture.repo_path("config/spaceship.yml");
    assert!(!spaceship.exists());

    // Act
    let result = fixture.handler.copy_to_repo();

    // Assert
    assert!(result.is_ok());
    assert!(tmuxconf.exists());
    assert!(spaceship.exists());

    let paths = [
        (true, "tmux.conf"),
        (true, "deepglob/config.yml"),
        (false, "ignored.txt"),
        (false, "diffed.txt.backup"),
        (false, "deepglob/test.out"),
        (false, "deepglob/.git/config"),
    ];

    for (exists, path) in paths {
        let p = fixture.repo_path(path);
        assert_eq!(exists, p.exists());
    }
}

#[test]
fn copy_to_home() {
    // Arrange
    let fixture = Fixture::setup();
    let envfile = fixture.home_path("env.toml");
    let diffedbackup = fixture.home_path("diffed.txt.backup");
    assert!(!envfile.exists());
    assert!(!diffedbackup.exists());

    // Act
    let result = fixture.handler.copy_to_home();

    // Assert
    assert!(result.is_ok());
    assert!(envfile.exists());
    assert!(diffedbackup.exists());
}
