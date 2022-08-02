#[derive(Debug)]
pub enum Item {
    Filepath(String),
    Object { path: String, name: Option<String> },
}
