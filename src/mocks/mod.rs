use crate::files::{Digester, FileHandler};
use crate::prompt::Prompt;
use anyhow::Result;
use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

pub type DigestFunc = dyn Fn(usize) -> String;

pub type Shared<T> = Rc<RefCell<T>>;

pub struct DigesterMock {
    count: RefCell<usize>,
    func: Box<DigestFunc>,
}

impl DigesterMock {
    pub fn new(count: RefCell<usize>, func: Box<DigestFunc>) -> Self {
        Self { count, func }
    }
}

impl Digester for DigesterMock {
    fn digest(&self, _data: &[u8]) -> Result<String> {
        let mut count = self.count.borrow_mut();
        let s = (self.func)(*count);
        *count = *count + 1;
        Ok(s)
    }
}

pub struct FileHandlerMock {
    content: Option<String>,
    created: Shared<Vec<String>>,
    copied: Shared<HashMap<String, String>>,
}

impl FileHandlerMock {
    pub fn new(created: Shared<Vec<String>>, copied: Shared<HashMap<String, String>>) -> Self {
        Self {
            created,
            copied,
            content: None,
        }
    }

    pub fn with_content(&mut self, content: String) {
        self.content = Some(content);
    }
}

impl FileHandler for FileHandlerMock {
    fn read_string(&self, _path: &Path) -> Result<String> {
        match &self.content {
            Some(s) => Ok(s.clone()),
            None => {
                let s = r#"{"test": true, "home": "~"}"#;
                Ok(s.to_string())
            }
        }
    }

    fn create_dirs(&self, path: &Path) -> Result<()> {
        let mut created = self.created.borrow_mut();
        let p = path.to_str().unwrap().to_string();
        created.push(p);
        Ok(())
    }

    fn copy(&self, src: &Path, dst: &Path) -> Result<()> {
        let mut copied = self.copied.borrow_mut();
        let src = src.to_str().unwrap().to_string();
        let dst = dst.to_str().unwrap().to_string();
        copied.insert(src, dst);
        Ok(())
    }
}

pub struct PromptMock;

impl Prompt for PromptMock {
    fn prompt(&self, _msg: &str) -> Result<String> {
        Ok("yes".to_string())
    }
}
