use crate::misc::{abs_path, path_set_named, regex_pattern};
use anyhow::Result;
use once_cell::sync::OnceCell;
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
    pub args: Option<Vec<Vec<String>>>,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, serde::Deserialize, Clone)]
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
    #[serde(default, with = "path_set_named")]
    pub learn: Vec<(String, PathBuf)>,
    #[serde(default, with = "path_set_named")]
    pub solve: Vec<(String, PathBuf)>,
}

static SUITE: OnceCell<Suite> = OnceCell::new();

#[allow(unused)]
pub fn get_suite() -> &'static Suite {
    SUITE.get().expect("suite not loaded")
}

pub fn load(path: &PathBuf) -> Result<()> {
    println!("suite path: {:?}", path);
    println!("suite dir: {:?}", path);
    let dir = env::current_dir()?;
    println!("reading suite...");
    let content = fs::read_to_string(path)?;
    println!("setting work dir to suite...");
    env::set_current_dir(path.parent().expect("suite is in no folder"))?;
    println!("parsing suite...");
    let suite: Suite = toml::from_str(&content)?;
    println!("restoring work dir...");
    env::set_current_dir(dir)?;
    for task in suite.tasks.iter() {
        if task.learn.is_empty() && task.solve.is_empty() {
            panic!("task {} has no problems", task.name);
        }
        if suite.learn.is_some() && task.learn.is_empty() {
            panic!("learner specified, but no learn problems in {}", task.name);
        }
        if suite.solve.is_some() && task.solve.is_empty() {
            panic!("solver specified, but no solve problems in {}", task.name);
        }
    }
    println!("task count: {}", suite.tasks.len());
    println!(
        "tasks: {:?}",
        suite
            .tasks
            .iter()
            .map(|t| &t.name)
            .collect::<Vec<&String>>()
    );
    println!(
        "problem learn count: {}",
        suite.tasks.iter().map(|t| t.learn.len()).sum::<usize>()
    );
    println!(
        "problem solve count: {}",
        suite.tasks.iter().map(|t| t.solve.len()).sum::<usize>()
    );
    let _ = SUITE.set(suite);
    Ok(())
}
