use crate::setup::instance::{Instance, RunKind};
use crate::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn collect(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    let mut file = File::create(out_dir.join("solve.csv"))
        .map_err(|e| format!("Failed to create file for solve data with error: {}", e))?;
    let _ = file.write(b"domain,problem,solver,exit_code\n");
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
        let exit_code = fs::read_to_string(run.dir.join("exit_code"))
            .map_err(|e| {
                format!(
                    "Failed to read exit code in {:?} with error: {}",
                    run.dir, e
                )
            })?
            .trim()
            .to_owned();
        let _ = file.write(format!("{},{},{},{}\n", domain, problem, solver, exit_code).as_bytes());
    }
    Ok(())
}
