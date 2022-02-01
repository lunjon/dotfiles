use super::*;
use crate::mocks::{DigestFunc, DigesterMock, FileHandlerMock, PromptMock, Shared};
use crate::utils::{FileSpec, TestContext};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct Setup {
    digest_func: Box<DigestFunc>,
}

struct Fixture {
    _context: TestContext,
    handler: Handler,
    _created_dirs: Shared<Vec<String>>,
    _copied: Shared<HashMap<String, String>>,
}

impl Fixture {}

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
            FileSpec {
                path: "what.vim".to_string(),
                status: Status::MissingHome,
            },
            FileSpec {
                path: "config/init.vim".to_string(),
                status: Status::Ok,
            },
            FileSpec {
                path: "config/spaceship.yml".to_string(),
                status: Status::Diff,
            },
        ]);
        context.setup().unwrap();

        let files = vec!["what.vim".to_string(), "config/".to_string()];

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
#[should_panic]
fn copy_to_repo_missing_source() {
    // Arrange
    let fixture = Setup::new().build();

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
