use crate::misc::path_set;
use crate::misc::regex_pattern;
use log::{info, trace, warn};
use regex::Regex;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Suite {
    pub name: String,
    pub time_limit: usize,
    pub memory_limit: usize,
    pub attributes: Vec<Attributes>,
    pub solvers: Vec<Solver>,
    pub tasks: Vec<Task>,
}

#[derive(Deserialize)]
pub struct Task {
    pub name: String,
    pub domain: PathBuf,
    #[serde(with = "path_set")]
    pub problems: Vec<PathBuf>,
}

#[derive(Deserialize)]
pub struct Solver {
    pub name: String,
    pub attributes: String,
    pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct Attributes {
    pub name: String,
    pub patterns: Vec<Pattern>,
}

#[derive(Deserialize)]
pub struct Pattern {
    pub name: String,
    #[serde(with = "regex_pattern")]
    pub pattern: Regex,
}

impl Suite {
    pub fn total_problems(&self) -> usize {
        self.tasks.iter().map(|s| s.problems.len()).sum::<usize>()
    }

    pub fn get_attributes(&self, name: &str) -> Option<&Attributes> {
        self.attributes.iter().find(|att| att.name == name)
    }
}

pub fn generate_suite(path: &PathBuf) -> Result<Suite, Box<dyn std::error::Error>> {
    trace!("Reading suite file at {:?}", &path);
    let suite_content = fs::read_to_string(path)?;
    trace!("Parsing suite");
    let suite: Suite = toml::from_str(&suite_content)?;
    info!("Suite name: {}", suite.name);
    info!("Suite time limit: {}s", suite.time_limit);
    info!("Suite memory limit: {}miB", suite.memory_limit);
    info!(
        "Solvers: {:?}",
        suite.solvers.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    info!(
        "Domains: {:?}",
        suite.tasks.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    info!("Total problems: {}", suite.total_problems());
    suite
        .tasks
        .iter()
        .filter(|t| t.problems.is_empty())
        .for_each(|t| warn!("Task {} contains 0 problems", &t.name));
    suite
        .solvers
        .iter()
        .filter(|t| suite.get_attributes(&t.attributes).is_none())
        .for_each(|s| {
            warn!(
                "Solver {} attributes {} doesn't exist",
                s.name, s.attributes
            )
        });
    Ok(suite)
}
