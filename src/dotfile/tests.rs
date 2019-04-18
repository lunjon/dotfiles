use super::*;
use crate::mocks::{DigestFunc, DigesterMock, FileHandlerMock, PromptMock, Shared};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct Setup {
    files: Vec<String>,
    home_dir: String,
    repo_dir: String,
    digest_func: Box<DigestFunc>,
}

struct Fixture {
    handler: Handler,
    created_dirs: Shared<Vec<String>>,
    copied: Shared<HashMap<String, String>>,
}

impl Fixture {
    fn assert_created(&self, dir: &str) {
        let dirs = self.created_dirs.borrow();
        let s = format!("{:#?}", dirs);

        assert!(
            dirs.iter().any(|d| d == dir),
            "expected {} to contain {}",
            s,
            dir
        );
    }

    fn assert_copied(&self, src: &str, dst: &str) {
        let copied = self.copied.borrow();
        let s = format!("{:#?}", copied);

        let entry = copied.get(src);
        assert!(entry.is_some(), "expected {} to contain key {}", s, src);

        let value = entry.unwrap();
        assert_eq!(value, dst);
    }
}

impl Setup {
    fn new(files: Vec<&str>) -> Self {
        Self {
            files: files.iter().map(|f| f.to_string()).collect(),
            home_dir: ".".to_string(),
            repo_dir: ".".to_string(),
            digest_func: Box::new(|count| "a".repeat(count + 1).to_string()),
        }
    }

    fn with_home(&mut self, path: &str) -> &mut Self {
        self.home_dir = path.to_string();
        self
    }

    fn with_repo(&mut self, path: &str) -> &mut Self {
        self.repo_dir = path.to_string();
        self
    }

    fn build(self) -> Fixture {
        // Used to track state
        let created_dirs = Rc::new(RefCell::new(Vec::new()));
        let copied = Rc::new(RefCell::new(HashMap::new()));

        let file_handler = Box::new(FileHandlerMock::new(
            Rc::clone(&created_dirs),
            Rc::clone(&copied),
        ));

        let digester = Box::new(DigesterMock::new(RefCell::new(0), self.digest_func));
        let prompt = Box::new(PromptMock {});

        let home = PathBuf::from(self.home_dir);
        let repo = PathBuf::from(self.repo_dir);
        let handler = Handler::new(file_handler, digester, prompt, home, repo, self.files);

        Fixture {
            handler,
            created_dirs,
            copied,
        }
    }
}

#[test]
fn copy_to_home() {
    // Arrange
    let mut setup = Setup::new(vec![".ideavimrc"]);
    setup.with_home(".").with_repo(".");
    let fixture = setup.build();

    // Act
    let result = fixture.handler.copy_to_home();

    // Assert
    assert!(result.is_ok());
    fixture.assert_copied("./files/.ideavimrc", "./.ideavimrc");
}

#[test]
fn copy_to_repo() {
    // Arrange
    let mut setup = Setup::new(vec!["src/bin/dotfiles.rs"]);
    setup.with_home(".").with_repo(".");
    let fixture = setup.build();

    // Act
    let result = fixture.handler.copy_to_repo();

    // Assert
    assert!(result.is_ok());
    fixture.assert_created("./files/src/bin");
    fixture.assert_copied("./src/bin/dotfiles.rs", "./files/src/bin/dotfiles.rs");
}

#[test]
fn make_entries_with_different_digest() {
    // Arrange
    let fixture = Setup::new(vec!["Cargo.toml"]).build();

    // Act
    let entries = fixture.handler.make_entries().unwrap();

    // Assert
    assert_eq!(1, entries.len());
}

#[test]
fn make_entries_single_file() {
    // Arrange
    let fixture = Setup::new(vec!["Cargo.toml"]).build();

    // Act
    let entries = fixture.handler.make_entries().unwrap();

    // Assert
    assert_eq!(1, entries.len());
}

#[test]
fn make_entries_single_directory() {
    let fixture = Setup::new(vec!["src/dotfile/"]).build();
    let entries = fixture.handler.make_entries().unwrap();
    assert_eq!(2, entries.len());
}

#[test]
fn make_entries_combo() {
    let fixture = Setup::new(vec!["src", "README.md"]).build();
    let entries = fixture.handler.make_entries().unwrap();
    assert_eq!(4, entries.len());
}

#[test]
fn make_entries_non_relative_file() {
    let fixture = Setup::new(vec!["/home/jonathan", "README.md"]).build();
    let entries = fixture.handler.make_entries().unwrap();
    assert_eq!(2, entries.len());

    let has_invalid = entries.iter().any(|e| e.is_invalid());
    assert!(has_invalid)
}

#[test]
fn make_entries_with_root_file() {
    let fixture = Setup::new(vec!["/etc/environment"]).build();
    let entries = fixture.handler.make_entries().unwrap();
    assert_eq!(1, entries.len());
    let has_invalid = entries.iter().any(|e| e.is_invalid());
    assert!(has_invalid)
}

#[test]
fn make_entries_with_root_directory() {
    let fixture = Setup::new(vec!["/usr/bin/"]).build();

    let entries = fixture.handler.make_entries().unwrap();
    assert_eq!(1, entries.len());
    let has_invalid = entries.iter().any(|e| e.is_invalid());
    assert!(has_invalid)
}

#[test]
fn get_status_missing_home() {
    let fixture = Setup::new(vec![]).build();
    let home_path = PathBuf::from(".zshrc");
    let repo_path = PathBuf::from("files/.zshrc");

    match fixture.handler.get_status(&home_path, &repo_path).unwrap() {
        Status::MissingHome => {}
        _ => panic!(),
    }
}

#[test]
fn get_status_missing_repo() {
    let fixture = Setup::new(vec![]).build();
    let home_path = PathBuf::from("Cargo.toml");
    let repo_path = PathBuf::from("files/Cargo.toml");

    match fixture.handler.get_status(&home_path, &repo_path).unwrap() {
        Status::MissingRepo => {}
        _ => panic!(),
    }
}

#[test]
fn get_status_missing_diff() {
    let fixture = Setup::new(vec![]).build();
    let home_path = PathBuf::from("Cargo.toml");
    let repo_path = PathBuf::from("files/.zshrc");

    match fixture.handler.get_status(&home_path, &repo_path).unwrap() {
        Status::Diff => {}
        _ => panic!(),
    }
}

#[test]
fn files_in_dir() {
    let path = PathBuf::from("src/");
    let result = Handler::files_in(&path);
    assert!(result.is_ok());

    let paths = result.unwrap();
    assert_eq!(3, paths.len());
}

#[test]
fn files_in_file() {
    let path = PathBuf::from("src/lib.rs");
    let result = Handler::files_in(&path);
    assert!(result.is_err());
}

#[test]
fn files_in_invalid() {
    let path = PathBuf::from("nonexistingfile.rs");
    let result = Handler::files_in(&path);
    assert!(result.is_err());
}
