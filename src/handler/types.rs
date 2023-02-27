use anyhow::{bail, Result};
use glob::Pattern as GlobPattern;
use regex::Regex;
use std::process::Command;

pub enum Pattern {
    Glob(GlobPattern),
    Regex(Regex),
}

impl Pattern {
    pub fn matches(&self, s: &str) -> bool {
        match self {
            Pattern::Glob(g) => g.matches(s),
            Pattern::Regex(g) => g.is_match(s),
        }
    }
}

pub struct Only {
    pub patterns: Vec<Pattern>,
}

impl Only {
    pub fn from_glob(patterns: &Vec<String>) -> Result<Self> {
        let mut ps = Vec::new();
        for p in patterns {
            let g = GlobPattern::new(p)?;
            ps.push(Pattern::Glob(g));
        }
        Ok(Self { patterns: ps })
    }

    pub fn from_regex(patterns: &Vec<String>) -> Result<Self> {
        let mut ps = Vec::new();
        for p in patterns {
            let r = Regex::new(p)?;
            ps.push(Pattern::Regex(r));
        }
        Ok(Self { patterns: ps })
    }
}

#[derive(Debug)]
pub struct DiffOptions {
    cmd: Vec<String>,
}

impl DiffOptions {
    pub fn new(cmd: Vec<String>) -> Self {
        Self { cmd }
    }

    pub fn to_cmd(&self, a: &str, b: &str) -> Result<Command> {
        let root = match self.cmd.get(0) {
            Some(r) => r,
            None => bail!("empty diff command"),
        };

        let mut cmd = Command::new(root);
        for arg in &self.cmd[1..] {
            cmd.arg(arg);
        }

        cmd.arg(a);
        cmd.arg(b);
        Ok(cmd)
    }
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            cmd: vec![
                String::from("diff"),
                String::from("-u"),
                String::from("--color"),
            ],
        }
    }
}
