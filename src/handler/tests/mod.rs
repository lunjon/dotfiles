use crate::data::Status;
use crate::prompt::Prompt;
use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

mod sync;

struct FileSpec {
    pub path: String,
    pub status: Status,
    // Used for creating e.g .git folders in home path.
    pub special: bool,
}

impl FileSpec {
    pub fn target(path: &str, status: Status) -> Self {
        Self {
            path: path.to_string(),
            status,
            special: false,
        }
    }

    pub fn special(path: &str) -> Self {
        Self {
            path: path.to_string(),
            status: Status::Ok,
            special: true,
        }
    }
}

fn random_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

fn create_with_path(path: &Path, content: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = fs::OpenOptions::new().write(true).create(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

struct TestContext {
    file_specs: Vec<FileSpec>,
    pub temp_dir: PathBuf,
    pub home_dir: PathBuf,
    pub repo_dir: PathBuf,
}

impl TestContext {
    pub fn new(file_specs: Vec<FileSpec>) -> Self {
        let s = format!("tmp-{}", random_string(10));
        let temp_dir = PathBuf::from(s);
        let mut home_dir = temp_dir.clone();
        home_dir.push("home");

        let mut repo_dir = temp_dir.clone();
        repo_dir.push("repo");

        Self {
            file_specs,
            temp_dir,
            home_dir,
            repo_dir,
        }
    }

    pub fn setup(&self) -> Result<()> {
        for spec in self.file_specs.iter() {
            if spec.special {
                let mut h = self.home_dir.clone();
                h.push(&spec.path);
                let content = random_string(10);
                create_with_path(&h, &content)?;
                continue;
            }

            match spec.status {
                Status::Ok => {
                    let mut h = self.home_dir.clone();
                    h.push(&spec.path);
                    let mut r = self.repo_dir.clone();
                    r.push(&spec.path);
                    let content = random_string(10);
                    create_with_path(&h, &content)?;
                    create_with_path(&r, &content)?;
                }
                Status::Diff => {
                    let mut h = self.home_dir.clone();
                    h.push(&spec.path);
                    let mut r = self.repo_dir.clone();
                    r.push(&spec.path);
                    let content = random_string(10);
                    create_with_path(&h, &content)?;
                    let content = random_string(10);
                    create_with_path(&r, &content)?;
                }
                Status::MissingHome => {
                    let mut r = self.repo_dir.clone();
                    r.push(&spec.path);
                    let content = random_string(10);
                    create_with_path(&r, &content)?;
                }
                Status::MissingRepo => {
                    let mut h = self.home_dir.clone();
                    h.push(&spec.path);
                    let content = random_string(10);
                    create_with_path(&h, &content)?;
                }
                Status::Invalid(_) => { /* Ignore */ }
            }
        }

        Ok(())
    }

    pub fn home_path(&self, path: &str) -> PathBuf {
        self.home_dir.join(path)
    }

    pub fn repo_path(&self, path: &str) -> PathBuf {
        self.repo_dir.join(path)
    }
}

impl Default for TestContext {
    fn default() -> Self {
        let specs = vec![
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
        ];
        Self::new(specs)
    }
}

impl Drop for TestContext {
    // Remove the tmp directory created for this fixture.
    fn drop(&mut self) {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).expect("failed to remove temporary test directory");
        }
    }
}

#[macro_export]
macro_rules! item {
    ($path:expr) => {
        crate::data::Item::Filepath(String::from($path))
    };
    ($path:expr, $($x:expr),*) => {
        {
            let mut tmp = Vec::new();
            $(
                tmp.push(String::from($x));
            )*

            crate::data::Item::Object {
                path: String::from($path),
                name: String::from($path),
                ignore: Some(tmp),
            }
        }
    };
}

#[test]
fn create_with_path_file() {
    // Act
    let path = PathBuf::from("new.rs");
    create_with_path(&path, "content").unwrap();

    // Assert
    assert!(path.exists());
    fs::remove_file(&path).unwrap();
}

#[test]
fn create_with_path_dir() {
    // Act
    let path = PathBuf::from("newdir/new.rs");
    create_with_path(&path, "content").unwrap();

    // Assert
    assert!(path.exists());
    fs::remove_dir_all("newdir").unwrap();
}

pub struct PromptMock;

impl Prompt for PromptMock {
    fn prompt(&self, _msg: &str) -> Result<String> {
        Ok("yes".to_string())
    }
}
