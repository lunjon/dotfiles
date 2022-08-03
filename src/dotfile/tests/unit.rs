use super::*;
use crate::dotfile::{Handler, Item::*, Status};
use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct FileSpec {
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

pub struct TestContext {
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
        let path = PathBuf::from(path);
        let mut new = self.home_dir.clone();
        new.push(&path);
        new
    }

    pub fn repo_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        let mut new = self.repo_dir.clone();
        new.push(&path);
        new
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

struct Fixture {
    _context: TestContext,
    handler: Handler,
    _created_dirs: Shared<Vec<String>>,
    _copied: Shared<HashMap<String, String>>,
}

struct Setup {
    digest_func: Box<DigestFunc>,
}

impl Setup {
    fn new() -> Self {
        Self {
            digest_func: Box::new(|count| "a".repeat(count + 1).to_string()),
        }
    }

    fn build(self) -> Fixture {
        let created_dirs = Rc::new(RefCell::new(Vec::new()));
        let copied = Rc::new(RefCell::new(HashMap::new()));

        let file_handler = Box::new(FileHandlerMock::new(
            Rc::clone(&created_dirs),
            Rc::clone(&copied),
        ));

        let digester = Box::new(DigesterMock::new(RefCell::new(0), self.digest_func));
        let prompt = Box::new(PromptMock {});

        let context = TestContext::new(vec![
            FileSpec::target("what.vim", Status::MissingHome),
            FileSpec::target("config/init.vim", Status::Ok),
            FileSpec::target("config/spaceship.yml", Status::Diff),
        ]);
        context.setup().unwrap();

        let files = vec![
            Filepath("what.vim".to_string()),
            Filepath("config/*".to_string()),
        ];

        let handler = Handler::new(
            file_handler,
            digester,
            prompt,
            context.home_dir.clone(),
            context.repo_dir.clone(),
            files,
        );

        Fixture {
            _context: context,
            handler,
            _created_dirs: created_dirs,
            _copied: copied,
        }
    }
}

#[test]
fn copy_to_home() {
    // Arrange
    let fixture = Setup::new().build();

    // Act
    fixture.handler.copy_to_home().unwrap();
}

#[test]
fn copy_to_repo() {
    // Arrange
    let mut fixture = Setup::new().build();
    fixture.handler.ignore_invalid(true);

    // Act
    fixture.handler.copy_to_repo().unwrap();
}

#[test]
fn make_entries() {
    // Arrange
    let fixture = Setup::new().build();

    // Act
    let entries = fixture.handler.make_entries().unwrap();

    // Assert
    assert_eq!(3, entries.len());
}

#[test]
fn get_status_missing_home() {
    let fixture = Setup::new().build();
    let home_path = PathBuf::from(".zshrc");
    let repo_path = PathBuf::from("files/.zshrc");

    match fixture.handler.get_status(&home_path, &repo_path).unwrap() {
        Status::MissingHome => {}
        _ => panic!(),
    }
}

#[test]
fn get_status_missing_repo() {
    let fixture = Setup::new().build();
    let home_path = PathBuf::from("Cargo.toml");
    let repo_path = PathBuf::from("files/Cargo.toml");

    match fixture.handler.get_status(&home_path, &repo_path).unwrap() {
        Status::MissingRepo => {}
        _ => panic!(),
    }
}

#[test]
fn get_status_diff() {
    let fixture = Setup::new().build();
    let home_path = PathBuf::from("Cargo.toml");
    let repo_path = PathBuf::from("README.md");

    let status = fixture.handler.get_status(&home_path, &repo_path).unwrap();
    assert!(matches!(status, Status::Diff));
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
