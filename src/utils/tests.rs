use super::*;
use crate::dotfile::Status;
use std::fs;
use std::path::PathBuf;

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

#[test]
fn testcontext_setup_drop() {
    let t: PathBuf;
    let h: PathBuf;
    let r: PathBuf;

    {
        let context = TestContext::new(vec![FileSpec {
            status: Status::Ok,
            path: "file.rs".to_string(),
        }]);
        t = context.temp_dir.clone();
        h = context.home_dir.clone();
        r = context.repo_dir.clone();

        context.setup().unwrap();
        assert!(t.exists());
        assert!(h.exists());
        assert!(r.exists());
    }

    assert!(!t.exists());
    assert!(!h.exists());
    assert!(!r.exists());
}

#[test]
fn testcontext_ok() {
    let context = TestContext::new(vec![FileSpec {
        path: "file.txt".to_string(),
        status: Status::Ok,
    }]);
    context.setup().unwrap();

    let home = context.home_path("file.txt");
    assert!(home.exists());
    let repo = context.repo_path("file.txt");
    assert!(repo.exists());
}

#[test]
fn testcontext_diff() {
    let context = TestContext::new(vec![FileSpec {
        path: "file.txt".to_string(),
        status: Status::Diff,
    }]);
    context.setup().unwrap();

    let home = context.home_path("file.txt");
    assert!(home.exists());
    let repo = context.repo_path("file.txt");
    assert!(repo.exists());
}

#[test]
fn testcontext_missing_home() {
    let context = TestContext::new(vec![FileSpec {
        path: "prefix/file.txt".to_string(),
        status: Status::MissingHome,
    }]);
    context.setup().unwrap();

    let home = context.home_path("prefix/file.txt");
    assert!(!home.exists());
    let repo = context.repo_path("prefix/file.txt");
    assert!(repo.exists());
}

#[test]
fn testcontext_missing_repo() {
    let context = TestContext::new(vec![FileSpec {
        path: "file.txt".to_string(),
        status: Status::MissingRepo,
    }]);
    context.setup().unwrap();

    let home = context.home_path("file.txt");
    assert!(home.exists());
    let repo = context.repo_path("file.txt");
    assert!(!repo.exists());
}

#[test]
fn testcontext_invalid() {
    let context = TestContext::new(vec![FileSpec {
        path: "file.txt".to_string(),
        status: Status::Invalid("invalid".to_string()),
    }]);
    context.setup().unwrap();

    let home = context.home_path("file.txt");
    assert!(!home.exists());
    let repo = context.repo_path("file.txt");
    assert!(!repo.exists());
}
