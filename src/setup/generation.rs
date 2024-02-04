use super::suite::Attributes;
use super::suite::Suite;
use log::info;
use log::trace;
use std::process::Command;
use std::{fs, path::PathBuf};

pub struct Instance<'a> {
    pub id: String,
    pub domain: &'a str,
    pub problem: &'a str,
    pub solver: &'a str,
    pub attributes: Option<&'a Attributes>,
    pub dir: PathBuf,
    pub exe: PathBuf,
}

fn generate_dir_iter(
    dir: &PathBuf,
    count: usize,
) -> impl Iterator<Item = std::string::String> + '_ {
    let width = ((count as f64).log10()).ceil() as usize;
    trace!("Width: {}", width);
    (0..count).into_iter().map(move |i| {
        let dir_name = format!("{:0>width$}", i, width = width);
        let _ = fs::create_dir_all(dir.join(&dir_name));
        dir_name
    })
}

pub fn generate_instances<'a>(
    memory_limit: usize,
    run_time: usize,
    working_dir: &PathBuf,
    suite: &'a Suite,
) -> Result<Vec<Instance<'a>>, Box<dyn std::error::Error>> {
    trace!("Generating working dir");
    let time_stamp = chrono::offset::Local::now().to_utc().to_string();
    trace!("Time stamp: {}", time_stamp);
    let working_dir = working_dir
        .join(&suite.name)
        .join(PathBuf::from(time_stamp));
    info!("Using working dir: {:?}", &working_dir);
    let instance_count = suite.solvers.len() * suite.total_problems();
    info!("Instance count: {}", instance_count);
    let mut dirs = generate_dir_iter(&working_dir, instance_count);

    let mut instances = vec![];
    for task in suite.tasks.iter() {
        for problem in task.problems.iter() {
            for solver in suite.solvers.iter() {
                let dir: PathBuf = dirs.next().unwrap().into();
                let dir: PathBuf = working_dir.join(dir);
                let runner = generate_runner(
                    memory_limit,
                    run_time,
                    &dir,
                    &solver.path,
                    &task.domain,
                    problem,
                )?;
                let id = dir.file_stem().unwrap().to_str().unwrap().to_owned();
                instances.push(Instance {
                    id,
                    domain: &task.name,
                    problem: problem.file_stem().unwrap().to_str().unwrap(),
                    solver: &solver.name,
                    attributes: suite.get_attributes(&solver.attributes),
                    dir,
                    exe: runner,
                });
            }
        }
    }
    Ok(instances)
}

fn generate_runner(
    memory_limit: usize,
    run_time: usize,
    dir: &PathBuf,
    solver_path: &PathBuf,
    domain: &PathBuf,
    problem: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(&dir)?;
    let runner = format!(
        "#!/bin/bash
ulimit -v {}
timeout {}s {} {:?} {:?} plan",
        memory_limit * 1000,
        run_time,
        solver_path.to_str().unwrap(),
        domain,
        problem
    );
    let runner_path = dir.join("runner.sh");
    fs::write(&runner_path, runner)?;
    let mut cmd = Command::new("chmod");
    cmd.arg("u+x");
    cmd.arg(&runner_path);
    cmd.status()?;
    Ok(runner_path)
}
