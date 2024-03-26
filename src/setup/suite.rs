use crate::misc::{abs_path, path_set, regex_pattern};
use log::{info, trace, warn};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Suite {
    pub name: String,
    pub time_limit_learn: Option<usize>,
    pub memory_limit_learn: Option<usize>,
    pub time_limit_solve: Option<usize>,
    pub memory_limit_solve: Option<usize>,
    pub attributes: Vec<Attributes>,
    pub learners: Vec<Learner>,
    pub solvers: Vec<Solver>,
    pub tasks: Vec<Task>,
}

#[derive(Deserialize)]
pub struct Task {
    pub name: String,
    #[serde(with = "abs_path")]
    pub domain: PathBuf,
    #[serde(with = "path_set")]
    pub problems_training: Vec<PathBuf>,
    #[serde(with = "path_set")]
    pub problems_testing: Vec<PathBuf>,
}

#[derive(Deserialize)]
pub struct Learner {
    pub name: String,
    pub attributes: String,
    #[serde(with = "abs_path")]
    pub path: PathBuf,
    pub args: Vec<String>,
}

#[derive(Deserialize)]
pub struct Solver {
    pub name: String,
    pub attributes: String,
    #[serde(with = "abs_path")]
    pub path: PathBuf,
    pub args: Vec<String>,
    pub learner: Option<String>,
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
    pub fn total_training_problems(&self) -> usize {
        self.tasks
            .iter()
            .map(|s| s.problems_training.len())
            .sum::<usize>()
    }

    pub fn total_testing_problems(&self) -> usize {
        self.tasks
            .iter()
            .map(|s| s.problems_testing.len())
            .sum::<usize>()
    }

    pub fn get_attributes(&self, name: &str) -> Option<&Attributes> {
        self.attributes.iter().find(|att| att.name == name)
    }
}

pub fn generate_suite(path: &PathBuf) -> Result<Suite, Box<dyn std::error::Error>> {
    trace!("Reading suite file at {:?}", &path);
    let suite_content = fs::read_to_string(path)?;
    let working_dir = env::current_dir().unwrap();
    let temp_dir = path.parent().unwrap();
    trace!("Setting working dir to {:?}", temp_dir);
    let _ = env::set_current_dir(temp_dir);
    trace!("Parsing suite");
    let suite: Suite = toml::from_str(&suite_content)?;
    trace!("Resetting dir to {:?}", working_dir);
    let _ = env::set_current_dir(working_dir);
    info!("Suite name: {}", suite.name);
    info!(
        "Solvers: {:?}",
        suite.solvers.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    info!(
        "Domains: {:?}",
        suite.tasks.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    info!(
        "Total training problems: {}",
        suite.total_training_problems()
    );
    info!("Total testing problems: {}", suite.total_testing_problems());
    suite
        .learners
        .iter()
        .filter(|t| suite.get_attributes(&t.attributes).is_none())
        .for_each(|s| {
            warn!(
                "Learner {} attributes {} doesn't exist",
                s.name, s.attributes
            )
        });
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
