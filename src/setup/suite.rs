use crate::misc::abs_path;
use crate::misc::path_set;
use crate::Result;
use log::info;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct Suite {
    pub time_limit_learn: Option<usize>,
    pub time_limit_solve: Option<usize>,
    pub memory_limit_learn: Option<usize>,
    pub memory_limit_solve: Option<usize>,
    pub runners: Vec<Runner>,
    pub tasks: Vec<Task>,
}

#[derive(serde::Deserialize)]
pub struct Runner {
    pub name: String,
    #[serde(with = "abs_path")]
    pub path: PathBuf,
    pub kind: RunnerKind,
    #[serde(default)]
    pub args: Vec<String>,
    pub depends: Option<String>,
}
#[derive(serde::Deserialize)]
pub enum RunnerKind {
    Learn,
    Solve,
}

#[derive(serde::Deserialize)]
pub struct Task {
    pub name: String,
    #[serde(with = "abs_path")]
    pub domain: PathBuf,
    #[serde(default, with = "path_set")]
    pub learn: Vec<PathBuf>,
    #[serde(default, with = "path_set")]
    pub solve: Vec<PathBuf>,
}

impl Suite {
    pub fn get_runner(&self, name: &str) -> Option<&Runner> {
        self.runners.iter().find(|r| r.name == name)
    }
    #[allow(unused)]
    pub fn get_task(&self, name: &str) -> Option<&Task> {
        self.tasks.iter().find(|r| r.name == name)
    }
    pub fn runner_names(&self) -> Vec<&str> {
        self.runners.iter().map(|r| r.name.as_str()).collect()
    }
    pub fn task_names(&self) -> Vec<&str> {
        self.tasks.iter().map(|r| r.name.as_str()).collect()
    }
    pub fn learner_count(&self) -> usize {
        self.learners().len()
    }
    pub fn solver_count(&self) -> usize {
        self.solvers().len()
    }
    pub fn total_problems_learn(&self) -> usize {
        self.tasks.iter().map(|t| t.learn.len()).sum()
    }
    pub fn total_problems_solve(&self) -> usize {
        self.tasks.iter().map(|t| t.solve.len()).sum()
    }
    pub fn learners(&self) -> Vec<&Runner> {
        self.runners
            .iter()
            .filter(|r| match r.kind {
                RunnerKind::Learn => true,
                RunnerKind::Solve => false,
            })
            .collect()
    }
    pub fn solvers(&self) -> Vec<&Runner> {
        self.runners
            .iter()
            .filter(|r| match r.kind {
                RunnerKind::Learn => false,
                RunnerKind::Solve => true,
            })
            .collect()
    }
}

pub fn parse(content: &str) -> Result<Suite> {
    let suite: Suite =
        toml::from_str(content).map_err(|e| format!("Failed to parse suite file: {}", e))?;

    // Checking whether any runner dependency is undefined
    for runner in suite.runners.iter() {
        if let Some(depends) = &runner.depends {
            if suite.get_runner(&depends).is_none() {
                return Err(format!(
                    "Runner {} depends on undefined runner {}",
                    runner.name, depends
                ));
            }
        }
    }

    // Checking whether tasks have problems according to the defined runners
    for task in suite.tasks.iter() {
        if task.learn.is_empty() && suite.learner_count() > 0 {
            return Err(format!("Task {} has no learn problems", task.name));
        }
        if task.solve.is_empty() && suite.solver_count() > 0 {
            return Err(format!("Task {} has no solve problems", task.name));
        }
        if task.learn.is_empty() && task.solve.is_empty() {
            return Err(format!("Task {} has no problems", task.name));
        }
    }

    info!("Runner count: {}", suite.runners.len());
    info!("Runners: {:?}", suite.runner_names());
    info!("Task count: {}", suite.tasks.len());
    info!("Tasks: {:?}", suite.task_names());
    info!("Problem learn count: {:?}", suite.total_problems_learn());
    info!("Problem solve count: {:?}", suite.total_problems_solve());

    Ok(suite)
}
