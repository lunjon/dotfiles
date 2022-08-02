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
    ///     repo/
    ///       diffed.txt      <- has diff
    ///       init.vim        <- no diff
    ///       env.toml        <- not in home
    ///
    /// The tmp-* directory is removed after each test.
    fn setup() -> Self {
        let context = TestContext::new(vec![
            FileSpec {
                path: "ignored.txt".to_string(),
                status: Status::MissingRepo,
            },
            FileSpec {
                path: "diffed.txt".to_string(),
                status: Status::Diff,
            },
            FileSpec {
                path: "init.vim".to_string(),
                status: Status::Ok,
            },
            FileSpec {
                path: "tmux.conf".to_string(),
                status: Status::MissingRepo,
            },
            FileSpec {
                path: "config/spaceship.yml".to_string(),
                status: Status::MissingRepo,
            },
            FileSpec {
                path: "env.toml".to_string(),
                status: Status::MissingHome,
            },
        ]);
        context.setup().unwrap();

        let file_handler = SystemFileHandler::default();
        let digester = Sha256Digest::default();

        // TODO: add object items as well
        let files = vec![
            Filepath("diffed.txt".to_string()),
            Filepath("tmux.conf".to_string()),
            Filepath("init.vim".to_string()),
            Filepath("config/".to_string()),
            Filepath("env.toml".to_string()),
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
