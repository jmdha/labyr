use crate::misc::{abs_path, path_set, regex_pattern};
use anyhow::Result;
use regex::Regex;
use std::{env, fs, path::PathBuf};

#[derive(Debug, serde::Deserialize)]
pub struct Suite {
    pub learn: Option<Experiment>,
    pub solve: Option<Experiment>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Experiment {
    pub time_limit: Option<usize>,
    pub memory_limit: Option<usize>,
    pub name: String,
    #[serde(with = "abs_path")]
    pub path: PathBuf,
    pub args: Option<Args>,
    pub attribute: Option<Attribute>,
}

#[derive(Debug, serde::Deserialize)]
pub enum Args {
    Single(Vec<String>),
    Zip(Vec<Vec<String>>),
    Cartesian(Vec<Vec<String>>),
}

#[derive(Debug, serde::Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(with = "regex_pattern")]
    pub pattern: Regex,
}

#[derive(Debug, serde::Deserialize)]
pub struct Task {
    pub name: String,
    #[serde(with = "abs_path")]
    pub domain: PathBuf,
    #[serde(default, with = "path_set")]
    pub learn: Vec<PathBuf>,
    #[serde(default, with = "path_set")]
    pub solve: Vec<PathBuf>,
}

pub fn load(path: &PathBuf) -> Result<Suite> {
    let dir = env::current_dir()?;
    let content = fs::read_to_string(path)?;
    env::set_current_dir(path.parent().expect("suite is in no folder"))?;
    let suite: Suite = toml::from_str(&content)?;
    env::set_current_dir(dir)?;
    Ok(suite)
}
