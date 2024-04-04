use super::suite::Suite;
use crate::Result;
use itertools::Itertools;
use log::trace;
use std::{env, fs, path::PathBuf, process::Command};

#[derive(Debug, Clone)]
pub struct Instance {
    pub runners: Vec<Runner>,
    pub tasks: Vec<Task>,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone)]
pub struct Runner {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub learn: Vec<String>,
    pub solve: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunKind {
    Learner,
    Solver {
        problem_index: usize,
        depends: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Run {
    pub dir: PathBuf,
    pub exe: PathBuf,
    pub runner_index: usize,
    pub task_index: usize,
    pub kind: RunKind,
}

pub fn generate(suite: Suite) -> Result<Instance> {
    let working_dir = env::current_dir().map_err(|e| format!("No working dir: {}", e))?;
    let learn_dir = working_dir.join("learn");
    let solve_dir = working_dir.join("solve");
    let mut runs: Vec<Run> = vec![];
    trace!("Instantiating tasks");
    for (i, ((task_index, task), (learner_index, learner))) in suite
        .tasks
        .iter()
        .enumerate()
        .cartesian_product(suite.learners().iter().enumerate())
        .enumerate()
    {
        let dir = learn_dir.join(format!("{}", i));
        let mut args = learner.args.clone();
        args.push(
            task.domain
                .to_str()
                .ok_or(format!("Failed to convert {:?} into a string", task.domain))?
                .to_owned(),
        );
        for problem in task.learn.iter() {
            args.push(
                problem
                    .to_str()
                    .ok_or(format!("Failed to convert {:?} into a string", problem))?
                    .to_owned(),
            );
        }
        let exe = generate_script(
            &dir,
            &learner.path,
            &args,
            suite.time_limit_learn,
            suite.memory_limit_learn,
        )?;
        runs.push(Run {
            dir,
            exe,
            runner_index: learner_index,
            task_index,
            kind: RunKind::Learner,
        });
    }
    let mut i: usize = 0;
    for (task_index, task) in suite.tasks.iter().enumerate() {
        for (problem_index, problem) in task.solve.iter().enumerate() {
            for (solver_index, solver) in suite.solvers().iter().enumerate() {
                let dir = solve_dir.join(format!("{}", i));
                let mut args = solver.args.clone();
                let depends = match &solver.depends {
                    Some(depends) => Some(
                        runs.iter()
                            .position(|l| {
                                task_index == l.task_index
                                    && depends == &suite.learners()[l.runner_index].name
                                    && l.kind == RunKind::Learner
                            })
                            .unwrap(),
                    ),
                    None => None,
                };
                if let Some(depends) = depends {
                    args.push(runs[depends].dir.to_string_lossy().to_string());
                }
                args.push(
                    task.domain
                        .to_str()
                        .ok_or(format!("Failed to convert {:?} into a string", task.domain))?
                        .to_owned(),
                );
                args.push(
                    problem
                        .to_str()
                        .ok_or(format!("Failed to convert {:?} into a string", problem))?
                        .to_owned(),
                );
                let exe = generate_script(
                    &dir,
                    &solver.path,
                    &args,
                    suite.time_limit_solve,
                    suite.memory_limit_solve,
                )?;
                runs.push(Run {
                    dir,
                    exe,
                    runner_index: solver_index + suite.learner_count(),
                    task_index,
                    kind: RunKind::Solver {
                        problem_index,
                        depends,
                    },
                });

                i += 1;
            }
        }
    }
    let runners = suite
        .runners
        .into_iter()
        .map(|r| Runner { name: r.name })
        .collect();
    let mut tasks = vec![];
    for task in suite.tasks.into_iter() {
        let name = task.name;
        let mut learn = vec![];
        let mut solve = vec![];

        for p in task.learn.into_iter() {
            learn.push(
                p.file_stem()
                    .ok_or(format!("Failed to get name of file {:?}", p))?
                    .to_str()
                    .ok_or(format!("Failed to convert path {:?} to string", p))?
                    .to_owned(),
            );
        }

        for p in task.solve.into_iter() {
            solve.push(
                p.file_stem()
                    .ok_or(format!("Failed to get name of file {:?}", p))?
                    .to_str()
                    .ok_or(format!("Failed to convert path {:?} to string", p))?
                    .to_owned(),
            );
        }

        tasks.push(Task { name, learn, solve })
    }
    Ok(Instance {
        runners,
        tasks,
        runs,
    })
}

fn generate_script(
    dir: &PathBuf,
    exe: &PathBuf,
    args: &Vec<String>,
    time_limit: Option<usize>,
    memory_limit: Option<usize>,
) -> Result<PathBuf> {
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create dir {:?} with error: {}", dir, e))?;
    let mut content = "#!/bin/bash\n".to_owned();
    if let Some(mem) = memory_limit {
        content.push_str(&format!("ulimit -v {}\n", mem * 1000));
    }
    let mut command: String = Default::default();
    if let Some(time) = time_limit {
        command.push_str(&format!("timeout {}s ", time));
    }
    command.push_str(&format!(
        "{} out{}",
        exe.to_str()
            .ok_or(format!("Failed to convert path {:?} to string", exe))?,
        args.iter()
            .map(|arg| format!(" {}", arg))
            .collect::<String>()
    ));
    content.push_str(&format!("$(eval \"{}\">log)\n", command));
    content.push_str(&format!("echo $? > exit_code"));
    let runner_path = dir.join("runner.sh");
    fs::write(&runner_path, content)
        .map_err(|e| format!("Failed to write script to file with error: {}", e))?;
    let mut cmd = Command::new("chmod");
    cmd.arg("u+x");
    cmd.arg(&runner_path);
    cmd.status()
        .map_err(|e| format!("Failed to give rights to script with error: {}", e))?;
    Ok(runner_path)
}
