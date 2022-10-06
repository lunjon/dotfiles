use super::PromptMock;
use super::TestContext;
use crate::data::Item;
use crate::handler::DiffOptions;
use crate::handler::{SyncHandler, SyncOptions};

fn string(name: &str, file: &str) -> Item {
    Item::from_str(name.to_string(), file.to_string())
}

fn object(name: &str, files: &[&str], ignore: Option<&[&str]>) -> Item {
    let ignore = ignore.map(|f| {
        f.to_vec()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    });
    let files = files
        .to_vec()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    Item::new(name.to_string(), files, ignore)
}

fn setup() -> (TestContext, SyncHandler) {
    let items = vec![
        string("diff", "diffed.txt"),
        string("tmux", "tmux.conf"),
        string("vim", "init.vim"),
        string("env", "env.toml"),
        string("conf", "config/*"),
        object("deep", &["deepglob/**/*"], Some(&["*.out"])),
    ];

    let context = TestContext::default();
    context.setup().unwrap();

    let options = SyncOptions {
        ignore_invalid: true,
        dryrun: false,
        confirm: false,
        backup: true,
        show_diff: false,
        diff_options: DiffOptions::default(),
    };

    let handler = SyncHandler::new(
        Box::new(PromptMock {}),
        context.home_dir.clone(),
        context.repo_dir.clone(),
        items,
        options,
        None,
    );

    (context, handler)
}

#[test]
fn copy_to_repo() {
    // Arrange
    let (context, handler) = setup();
    let tmuxconf = context.repo_path("tmux.conf");
    assert!(!tmuxconf.exists());
    let spaceship = context.repo_path("config/spaceship.yml");
    assert!(!spaceship.exists());

    // Act
    let result = handler.copy_to_repo();

    // Assert
    assert!(result.is_ok());
    assert!(spaceship.exists());

    let paths = [
        (true, "tmux.conf"),
        (true, "deepglob/config.yml"),
        (false, "ignored.txt"),
        (false, "diffed.txt.backup"),
        (false, "deepglob/test.out"),
        (false, "deepglob/.git/config"),
    ];

    for (exists, path) in paths {
        let p = context.repo_path(path);
        assert_eq!(exists, p.exists());
    }
}

#[test]
fn copy_to_home() {
    // Arrange
    let (context, handler) = setup();
    let envfile = context.home_path("env.toml");
    let diffedbackup = context.home_path("diffed.txt.backup");
    assert!(!envfile.exists());
    assert!(!diffedbackup.exists());

    // Act
    let result = handler.copy_to_home();

    // Assert
    assert!(result.is_ok());
    assert!(envfile.exists());
    assert!(diffedbackup.exists());
}
