use super::*;

#[test]
fn digest_empty() {
    let digester = Sha256Digest {};
    let d = digester.digest("".as_bytes());
    assert!(d.is_ok());
}

#[test]
fn digest_content() {
    let digester = Sha256Digest {};
    let d = digester.digest("content".as_bytes());
    assert!(d.is_ok());
}
