use crate::setup::instance::{Instance, RunKind, Runner};
use crate::setup::suite::{Attribute, RunnerKind};
use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use super::{pattern_names, pattern_values};

pub fn collect(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    let mut file = File::create(out_dir.join("solve.csv"))?;
    let solve_runners = instance
        .runners
        .iter()
        .filter(|r| r.kind == RunnerKind::Solve)
        .collect::<Vec<&Runner>>();
    let attributes = solve_runners
        .iter()
        .filter_map(|r| match r.attribute {
            Some(a) => Some(&instance.attributes[a]),
            None => None,
        })
        .collect::<Vec<&Attribute>>();
    let pattern_names = pattern_names(attributes);
    let _ = file.write(b"domain,problem,name,exit_code");
    if !pattern_names.is_empty() {
        let _ = file.write(format!(",{}", pattern_names.join(",")).as_bytes());
    }
    let _ = file.write(b"\n");
    for (run, problem) in instance.runs.iter().filter_map(|r| match r.kind {
        RunKind::Learner => None,
        RunKind::Solver {
            problem_index: i,
            depends: _,
        } => Some((r, i)),
    }) {
        let solver = &instance.runners[run.runner_index].name;
        let domain = &instance.tasks[run.task_index].name;
        let problem = &instance.tasks[run.task_index].solve[problem];
        let exit_code = fs::read_to_string(run.dir.join("exit_code"))?
            .trim()
            .to_owned();
        let _ = file.write(format!("{},{},{},{}", domain, problem, solver, exit_code).as_bytes());
        if let Some(attribute) = instance.runners[run.runner_index].attribute {
            let content = fs::read_to_string(run.dir.join("log"))?;
            let p_values =
                pattern_values(&pattern_names, &instance.attributes[attribute], &content);
            let _ = file.write(format!(",{}", p_values.join(",")).as_bytes());
        }
        let _ = file.write(b"\n");
    }
    Ok(())
}
