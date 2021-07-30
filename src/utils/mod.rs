use rand::{distributions::Alphanumeric, Rng};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn random_tmp_dir(size: usize) -> PathBuf {
    let r = rand::thread_rng();
    let s: String = r
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect();
    PathBuf::from(format!("tmp-{}", s))
}

pub fn create_file(path: &Path, content: &str) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("create new file");
    file.write_all(content.as_bytes()).expect("write file");
}
