use anyhow::Result;
use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use std::fs;
use std::io::Read;
use std::path::Path;

#[cfg(test)]
mod tests;

pub trait Digester {
    fn digest(&self, data: &[u8]) -> Result<String> {
        let mut context = Context::new(&SHA256);
        context.update(data);

        let digest = context.finish();
        let s = HEXLOWER.encode(digest.as_ref());
        Ok(s)
    }
}

#[derive(Default)]
pub struct Sha256Digest;

impl Digester for Sha256Digest {}

pub trait FileHandler {
    fn copy(&self, src: &Path, dst: &Path) -> Result<()>;

    fn read_string(&self, path: &Path) -> Result<String>;

    fn create_dirs(&self, path: &Path) -> Result<()>;
}

#[derive(Default)]
pub struct SystemFileHandler;

impl FileHandler for SystemFileHandler {
    fn copy(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::copy(&src, &dst)?;
        Ok(())
    }

    fn read_string(&self, path: &Path) -> Result<String> {
        let mut buf = String::new();
        let mut file = fs::File::open(&path)?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    fn create_dirs(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(&path)?;
        Ok(())
    }
}
