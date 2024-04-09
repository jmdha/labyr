use crate::setup::instance::{Instance, RunKind, Runner};
use crate::setup::suite::{Attribute, RunnerKind};
use crate::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use super::{pattern_names, pattern_values};

pub fn collect(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    let mut file = File::create(out_dir.join("learn.csv"))
        .map_err(|e| format!("Failed to create file for solve data with error: {}", e))?;
    let learn_runners = instance
        .runners
        .iter()
        .filter(|r| r.kind == RunnerKind::Learn)
        .collect::<Vec<&Runner>>();
    let attributes = learn_runners
        .iter()
        .filter_map(|r| match r.attribute {
            Some(a) => Some(&instance.attributes[a]),
            None => None,
        })
        .collect::<Vec<&Attribute>>();
    let pattern_names = pattern_names(attributes);
    let _ = file.write(b"domain,learner,exit_code");
    if !pattern_names.is_empty() {
        let _ = file.write(format!(",{}", pattern_names.join(",")).as_bytes());
    }
    let _ = file.write(b"\n");
    for run in instance.runs.iter().filter(|r| match r.kind {
        RunKind::Learner => true,
        RunKind::Solver {
            problem_index: _,
            depends: _,
        } => false,
    }) {
        let learner = &instance.runners[run.runner_index].name;
        let domain = &instance.tasks[run.task_index].name;
        let exit_code = fs::read_to_string(run.dir.join("exit_code"))
            .map_err(|e| {
                format!(
                    "Failed to read exit code in {:?} with error: {}",
                    run.dir, e
                )
            })?
            .trim()
            .to_owned();
        let _ = file.write(format!("{},{},{}", domain, learner, exit_code).as_bytes());
        if let Some(attribute) = instance.runners[run.runner_index].attribute {
            let content = fs::read_to_string(run.dir.join("log")).map_err(|e| {
                format!("Failed to read run log in {:?} with error: {}", run.dir, e)
            })?;
            let p_values =
                pattern_values(&pattern_names, &instance.attributes[attribute], &content);
            let _ = file.write(format!(",{}", p_values.join(",")).as_bytes());
        }
        let _ = file.write(b"\n");
    }
    Ok(())
}
