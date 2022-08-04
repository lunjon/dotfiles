use anyhow::Result;
use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use std::fs;
use std::io::Read;
use std::path::Path;

#[cfg(test)]
mod tests;

pub fn digest(data: &[u8]) -> Result<String> {
    let mut context = Context::new(&SHA256);
    context.update(data);

    let digest = context.finish();
    let s = HEXLOWER.encode(digest.as_ref());
    Ok(s)
}

pub fn copy(src: &Path, dst: &Path) -> Result<()> {
    fs::copy(&src, &dst)?;
    Ok(())
}

pub fn read_string(path: &Path) -> Result<String> {
    let mut buf = String::new();
    let mut file = fs::File::open(&path)?;
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn create_dirs(path: &Path) -> Result<()> {
    fs::create_dir_all(&path)?;
    Ok(())
}
