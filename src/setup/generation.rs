use super::suite::{Attributes, Suite};
use itertools::Itertools;
use log::{info, trace};
use std::{error::Error, fs, path::PathBuf, process::Command};

pub enum InstanceKind<'a> {
    Learner,
    Solver { problem: &'a str },
}

pub struct Instance<'a> {
    pub name: &'a str,
    pub exe: PathBuf,
    pub dir: PathBuf,
    pub attributes: Option<&'a Attributes>,
    pub domain: &'a str,
    pub kind: InstanceKind<'a>,
}

pub struct Instances<'a> {
    pub learners: Vec<Instance<'a>>,
    pub solvers: Vec<Instance<'a>>,
}

pub fn generate_instances<'a>(
    memory_limit: usize,
    run_time: usize,
    working_dir: &PathBuf,
    suite: &'a Suite,
) -> Result<Instances<'a>, Box<dyn Error>> {
    trace!("Generating working dir");
    let time_stamp = chrono::offset::Local::now().to_utc().to_string();
    let time_stamp = time_stamp.replace(" ", "");
    trace!("Time stamp: {}", time_stamp);
    let working_dir = working_dir
        .join(&suite.name)
        .join(PathBuf::from(time_stamp));
    info!("Using working dir: {:?}", &working_dir);
    let learner_dir = working_dir.join("learner");
    let solver_dir = working_dir.join("solver");
    let learners = generate_learners(memory_limit, run_time, &learner_dir, suite)?;
    let solvers = generate_solvers(memory_limit, run_time, &solver_dir, suite, &learners)?;
    Ok(Instances { learners, solvers })
}

fn generate_learners<'a>(
    memory_limit: usize,
    run_time: usize,
    working_dir: &PathBuf,
    suite: &'a Suite,
) -> Result<Vec<Instance<'a>>, Box<dyn Error>> {
    trace!("Generating learners");
    let mut instances = vec![];
    for (i, (task, learner)) in suite
        .tasks
        .iter()
        .cartesian_product(suite.learners.iter())
        .enumerate()
    {
        let dir = working_dir.join(format!("{}", i));
        let mut args = learner.args.to_owned();
        args.push(task.domain.to_str().unwrap().to_owned());
        for problem in task.problems_training.iter() {
            args.push(problem.to_str().unwrap().to_owned());
        }
        let runner = generate_runner(memory_limit, run_time, &dir, &learner.path, &args)?;
        instances.push(Instance {
            name: &learner.name,
            exe: runner,
            dir,
            attributes: suite.get_attributes(&learner.attributes),
            domain: &task.name,
            kind: InstanceKind::Learner,
        });
    }
    Ok(instances)
}

fn generate_solvers<'a>(
    memory_limit: usize,
    run_time: usize,
    working_dir: &PathBuf,
    suite: &'a Suite,
    learners: &Vec<Instance<'a>>,
) -> Result<Vec<Instance<'a>>, Box<dyn Error>> {
    trace!("Generating solvers");
    let mut instances = vec![];
    let mut i = 0;
    for (task, solver) in suite.tasks.iter().cartesian_product(suite.solvers.iter()) {
        let learner = match &solver.learner {
            Some(learner) => Some(
                learners
                    .iter()
                    .position(|l| learner == l.name && task.name == l.domain)
                    .unwrap(),
            ),
            None => None,
        };
        for problem in task.problems_testing.iter() {
            let dir = working_dir.join(format!("{}", i));
            let mut args = vec![];
            if let Some(learner) = learner {
                args.push(learners[learner].dir.to_str().unwrap().to_owned());
            }
            args.append(&mut solver.args.to_owned());
            args.push(task.domain.to_str().unwrap().to_owned());
            args.push(problem.to_str().unwrap().to_owned());
            let runner = generate_runner(memory_limit, run_time, &dir, &solver.path, &args)?;
            instances.push(Instance {
                name: &solver.name,
                exe: runner,
                dir,
                attributes: suite.get_attributes(&solver.attributes),
                domain: &task.name,
                kind: InstanceKind::Solver {
                    problem: problem.to_str().unwrap(),
                },
            });
            i += 1;
        }
    }
    Ok(instances)
}

fn generate_runner(
    memory_limit: usize,
    run_time: usize,
    dir: &PathBuf,
    exe: &PathBuf,
    args: &Vec<String>,
) -> Result<PathBuf, Box<dyn Error>> {
    fs::create_dir_all(&dir)?;
    let content = format!(
        "#!/bin/bash
ulimit -m {}
timeout {}s {} plan{}",
        memory_limit * 1000,
        run_time,
        exe.to_str().unwrap(),
        args.iter()
            .map(|arg| format!(" {}", arg))
            .collect::<String>()
    );
    let runner_path = dir.join("runner.sh");
    fs::write(&runner_path, content)?;
    let mut cmd = Command::new("chmod");
    cmd.arg("u+x");
    cmd.arg(&runner_path);
    cmd.status()?;
    Ok(runner_path)
}
