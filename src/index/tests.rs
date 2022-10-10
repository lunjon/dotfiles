use crate::testing::TestContext;

use super::*;

#[test]
fn should_ignore_true() {
    let patterns = &[
        GlobPattern::new("*.txt").unwrap(),
        GlobPattern::new(".git/**/*").unwrap(),
    ];
    let paths = &[
        "root.txt",
        "path/test.txt",
        ".git/test",
        ".git/hooks/commit",
    ];

    for path in paths {
        assert!(should_ignore(path, patterns));
    }
}

// Setup test context and indexer.
// NOTE! Make sure that context is also stored, else it is dropped directly.
// That is, use it like this:
//     let (_cx, indexer) = setup();

fn setup() -> (TestContext, Indexer) {
    let cx = TestContext::default();
    cx.setup().expect("to setup test context");
    let indexer = Indexer::new(cx.home_dir.clone(), cx.repo_dir.clone(), None);
    (cx, indexer)
}

#[test]
fn return_zero_entries_given_empty_list() {
    let (_, indexer) = setup();
    let res = indexer.index(&vec![]).unwrap();
    assert!(res.is_empty());
}

#[test]
fn return_entry_given_one() {
    // Arrange
    let (_cx, indexer) = setup();
    let items = vec![Item::simple_new("diff", "diffed.txt")];
    // Act
    let indexed = indexer.index(&items).unwrap();

    // Assert
    assert_eq!(1, indexed.len());
    let (name, entries) = indexed.first().expect("to get first");
    assert_eq!("diff", name);
    assert_eq!(1, entries.len());
}

#[test]
fn return_entry_in_directory() {
    // Arrange
    let (_cx, indexer) = setup();
    let items = vec![Item::simple_new("space", "config/spaceship.yml")];
    // Act
    let indexed = indexer.index(&items).unwrap();

    // Assert
    assert_eq!(1, indexed.len());
    let (_, entries) = indexed.first().expect("to get first");
    let contains_file = entries
        .iter()
        .any(|e| e.get_relpath().contains("spaceship"));
    assert!(contains_file);
}

#[test]
fn respect_ignore() {
    // Arrange
    let (_cx, indexer) = setup();
    let items = vec![Item::object_new(
        "glob",
        &["deepglob/src/*"],
        Some(&["*lock.json"]),
    )];

    // Act
    let indexed = indexer.index(&items).unwrap();

    // Assert
    assert_eq!(1, indexed.len());
    let (_, entries) = indexed.first().expect("to get first");
    assert_eq!(2, entries.len());
}

#[test]
fn ignores_special_directories() {
    // Arrange
    let (_cx, indexer) = setup();
    let items = vec![Item::object_new("glob", &["deepglob/**/*"], None)];
    // Act
    let indexed = indexer.index(&items).unwrap();

    // Assert
    assert_eq!(1, indexed.len());
    let (_, entries) = indexed.first().expect("to get first");
    let contains_git = entries
        .iter()
        .any(|entry| entry.get_relpath().contains(".git"));
    assert!(!contains_git);
}
