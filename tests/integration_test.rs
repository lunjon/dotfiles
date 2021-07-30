use anyhow::Result;
use dotfiles::dotfile::Handler;
use dotfiles::files::{Sha256Digest, SystemFileHandler};
use dotfiles::prompt::Prompt;
use dotfiles::utils;
use std::fs;
use std::path::PathBuf;

pub struct PromptMock;

impl Prompt for PromptMock {
    fn prompt(&self, _msg: &str) -> Result<String> {
        Ok("yes".to_string())
    }
}

struct Fixture {
    handler: Handler,
    tmp_dir: PathBuf,
    home_dir: PathBuf,
    repo_dir: PathBuf,
}

impl Fixture {
    /// Setup for integration tests by creating actual files.
    /// The structure looks like:
    ///   tmp-*/
    ///     home/
    ///       ignored.txt       <- ignored
    ///       init.vim          <- no diff
    ///       tmux.conf         <- new file
    ///       config/
    ///         spaceship.yml   <- new file
    ///     repo/
    ///       files/
    ///         init.vim        <- no diff
    ///         env.toml        <- not in home
    ///
    /// The tmp-* directory is removed after each test.
    fn setup() -> Self {
        // Create directories
        let tmp_dir = utils::random_tmp_dir(8);
        let mut home_dir = tmp_dir.clone();
        home_dir.push("home");

        let mut config = home_dir.clone();
        config.push("config");

        let mut repo_dir = tmp_dir.clone();
        repo_dir.push("repo");

        fs::create_dir_all(&config).expect("failed to create dir");
        fs::create_dir_all(&repo_dir).expect("failed to create dir");

        // Create files
        let mut path = home_dir.clone();
        path.push("ignored.txt");
        utils::create_file(&path, "not included");

        let mut path = home_dir.clone();
        path.push("init.vim");
        utils::create_file(&path, "set number");
        let mut dst = repo_dir.clone();
        dst.push("init.vim");
        fs::copy(&path, &dst).expect("failed to copy file");

        let mut path = home_dir.clone();
        path.push("tmux.conf");
        utils::create_file(&path, "set -g default-terminal /home/user/.cargo/bin/nu");

        let mut path = config.clone();
        path.push("spaceship.yml");
        let spaceship = "version: 1
files:
  - one
  - two
";
        utils::create_file(&path, spaceship);

        let mut path = repo_dir.clone();
        path.push("env.toml");
        utils::create_file(&path, "value = true");

        let file_handler = SystemFileHandler::default();
        let digester = Sha256Digest::default();

        let files = vec![
            "tmux.conf".to_string(),
            "init.vim".to_string(),
            "config/".to_string(),
            "env.toml".to_string(),
        ];

        let mut handler = Handler::new(
            Box::new(file_handler),
            Box::new(digester),
            Box::new(PromptMock {}),
            home_dir.clone(),
            repo_dir.clone(),
            files,
        );
        handler.ignore_invalid(true);

        Self {
            tmp_dir,
            handler,
            home_dir,
            repo_dir,
        }
    }

    fn home_path(&self, paths: Vec<&str>) -> PathBuf {
        let mut path = self.home_dir.clone();
        for p in &paths {
            path.push(p);
        }
        path
    }

    fn repo_path(&self, paths: Vec<&str>) -> PathBuf {
        let mut path = self.repo_dir.clone();
        for p in &paths {
            path.push(p);
        }
        path
    }
}

impl Drop for Fixture {
    // Remove the tmp directory created for this fixture.
    fn drop(&mut self) {
        fs::remove_dir_all(&self.tmp_dir).expect("failed to remove temporary test directory");
    }
}

#[test]
fn copy_to_repo() {
    // Arrange
    let fixture = Fixture::setup();
    let tmuxconf = fixture.repo_path(vec!["tmux.conf"]);
    assert!(!tmuxconf.exists());
    let spaceship = fixture.repo_path(vec!["config", "spaceship.yml"]);
    assert!(!spaceship.exists());

    // Act
    let result = fixture.handler.copy_to_repo();

    // Assert
    assert!(result.is_ok());
    assert!(tmuxconf.exists());
    assert!(spaceship.exists());

    let ignored = fixture.repo_path(vec!["ignored.txt"]);
    assert!(!ignored.exists());
}

#[test]
fn copy_to_home() {
    // Arrange
    let fixture = Fixture::setup();
    let envfile = fixture.home_path(vec!["env.toml"]);
    assert!(
        !envfile.exists(),
        "env.toml should not exists before copy_to_home()"
    );

    // Act
    let result = fixture.handler.copy_to_home();

    // Assert
    assert!(result.is_ok());
    assert!(
        envfile.exists(),
        "env.toml should exists after copy_to_home()"
    );
}
