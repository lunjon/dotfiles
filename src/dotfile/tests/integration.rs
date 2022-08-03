use super::unit::*;
use crate::dotfile::{Handler, Item::*, Status};
use crate::files::{Sha256Digest, SystemFileHandler};
use crate::prompt::Prompt;
use anyhow::Result;
use std::path::PathBuf;

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
    ///       init.vim          <- no diff
    ///       tmux.conf         <- new file
    ///       config/
    ///         spaceship.yml   <- new file
    ///         .git            <- should be ignored
    ///       deepglob/
    ///         config.yml      <- not in repo
    ///         node_modules    <- should be ignored
    ///         .git            <- should be ignored
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
            FileSpec::special("deepglob/.git/config"),
            FileSpec::special("deepglob/.git/objects/abc123"),
            FileSpec::special("deepglob/.git/objects/def456"),
        ]);

        context.setup().unwrap();
        let file_handler = SystemFileHandler::default();
        let digester = Sha256Digest::default();

        let files = vec![
            Filepath("diffed.txt".to_string()),
            Filepath("tmux.conf".to_string()),
            Filepath("init.vim".to_string()),
            Filepath("env.toml".to_string()),
            Object {
                path: "config/*".to_string(),
                name: None,
            },
            Object {
                path: "deepglob/**/*".to_string(),
                name: Some("globber".to_string()),
            },
        ];

        let mut handler = Handler::new(
            Box::new(file_handler),
            Box::new(digester),
            Box::new(PromptMock {}),
            context.home_dir.clone(),
            context.repo_dir.clone(),
            files,
        );
        handler.ignore_invalid(true);

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
    let ignored = fixture.repo_path("ignored.txt");
    assert!(!ignored.exists());
    let config_file = fixture.repo_path("deepglob/config.yml");
    assert!(config_file.exists());
    let git_file = fixture.repo_path("deepglob/.git/config");
    assert!(!git_file.exists());
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
