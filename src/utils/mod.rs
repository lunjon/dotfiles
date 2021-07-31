use crate::dotfile::Status;
use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

pub fn random_string(size: usize) -> String {
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

pub struct FileSpec {
    pub path: String,
    pub status: Status,
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
            match spec.status {
                Status::Ok => {
                    let path = PathBuf::from(&spec.path);
                    let mut h = self.home_dir.clone();
                    h.push(&path);
                    let mut r = self.repo_dir.clone();
                    r.push(&path);
                    let content = random_string(10);
                    create_with_path(&h, &content)?;
                    create_with_path(&r, &content)?;
                }
                Status::Diff => {
                    let path = PathBuf::from(&spec.path);
                    let mut h = self.home_dir.clone();
                    h.push(&path);
                    let mut r = self.repo_dir.clone();
                    r.push(&path);
                    let content = random_string(10);
                    create_with_path(&h, &content)?;
                    let content = random_string(10);
                    create_with_path(&r, &content)?;
                }
                Status::MissingHome => {
                    let path = PathBuf::from(&spec.path);
                    let mut r = self.repo_dir.clone();
                    r.push(&path);
                    let content = random_string(10);
                    create_with_path(&r, &content)?;
                }
                Status::MissingRepo => {
                    let path = PathBuf::from(&spec.path);
                    let mut h = self.home_dir.clone();
                    h.push(&path);
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
