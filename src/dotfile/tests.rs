use super::*;
use crate::mocks::{DigestFunc, DigesterMock, FileHandlerMock, PromptMock, Shared};
use crate::utils;
use std::fs;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct Setup {
    files: Vec<String>,
    digest_func: Box<DigestFunc>,
}

struct Fixture {
    repo: PathBuf,
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

    fn temp_dir(&self) -> String {
        self.repo.to_str().unwrap().to_string()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.repo).unwrap();
    }
}

impl Setup {
    fn new(files: Vec<&str>) -> Self {
        Self {
            files: files.iter().map(|f| f.to_string()).collect(),
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

        let home = PathBuf::from(".");

        // Create temporary directory for the repo path
        // and create a file called what.vim.
        let repo = utils::random_tmp_dir(8);
        fs::create_dir(&repo).unwrap();
        let mut what = repo.clone();
        what.push("what.vim");
        utils::create_file(&what, r"What");

        let handler = Handler::new(
            file_handler,
            digester,
            prompt,
            home,
            repo.clone(),
            self.files,
        );

        Fixture {
            repo,
            handler,
            created_dirs,
            copied,
        }
    }
}

#[test]
fn copy_to_home() {
    // Arrange
    let fixture = Setup::new(vec!["what.vim"]).build();

    // Act
    let result = fixture.handler.copy_to_home();

    // Assert
    assert!(result.is_ok());
    let src = format!("{}/what.vim", fixture.temp_dir());
    fixture.assert_copied(&src, "./what.vim");
}

#[test]
fn copy_to_repo() {
    // Arrange
    let fixture = Setup::new(vec!["src/bin/dotfiles.rs"]).build();

    // Act
    let result = fixture.handler.copy_to_repo();

    // Assert
    assert!(result.is_ok());
    let dst = format!("{}/src/bin", fixture.temp_dir());
    fixture.assert_created(&dst);
    let dst = format!("{}/src/bin/dotfiles.rs", fixture.temp_dir());
    fixture.assert_copied("./src/bin/dotfiles.rs", &dst);
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
    let repo_path = PathBuf::from(format!("{}/what.vim", fixture.temp_dir()));

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
