use super::digest;

#[test]
fn digest_empty() {
    let d = digest("".as_bytes());
    assert!(d.is_ok());
}

#[test]
fn digest_content() {
    let d = digest("content".as_bytes());
    assert!(d.is_ok());
}
