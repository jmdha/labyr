use super::suite::{Attribute, RunnerKind, Suite};
use anyhow::Result;
use log::trace;
use std::{env, fs, path::PathBuf, process::Command};

#[derive(Debug, Clone)]
pub struct Instance {
    pub learn_dir: PathBuf,
    pub solve_dir: PathBuf,
    pub learn_mem_limit: Option<usize>,
    pub solve_mem_limit: Option<usize>,
    pub runners: Vec<Runner>,
    pub tasks: Vec<Task>,
    pub attributes: Vec<Attribute>,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone)]
pub struct Runner {
    pub name: String,
    pub attribute: Option<usize>,
    pub kind: RunnerKind,
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
    let working_dir = env::current_dir()?;
    let learn_dir = working_dir.join("learn");
    let solve_dir = working_dir.join("solve");
    let mut runs: Vec<Run> = vec![];
    trace!("Instantiating tasks");
    let mut i: usize = 0;
    for (task_index, task) in suite.tasks.iter().enumerate() {
        for (learner_index, learner) in suite.learners().iter().enumerate() {
            let dir = learn_dir.join(format!("{}", i));
            let mut args = learner.args.clone();
            args.push(task.name.to_owned());
            args.push(task.domain.to_string_lossy().to_string());
            for problem in task.learn.iter() {
                args.push(problem.to_string_lossy().to_string());
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
            i += 1;
        }
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
                args.push(task.domain.to_string_lossy().to_string());
                args.push(problem.to_string_lossy().to_string());
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
    let attributes = suite.attributes;
    let runners = suite
        .runners
        .into_iter()
        .map(|r| Runner {
            name: r.name,
            attribute: match r.attribute {
                Some(a) => Some(attributes.iter().position(|p| p.name == a).unwrap()),
                None => None,
            },
            kind: r.kind,
        })
        .collect();
    let mut tasks = vec![];
    for task in suite.tasks.into_iter() {
        let name = task.name;
        let mut learn = vec![];
        let mut solve = vec![];

        for p in task.learn.into_iter() {
            learn.push(
                p.file_stem()
                    .expect(&format!("problem {:?} has no name", p))
                    .to_string_lossy()
                    .to_string(),
            );
        }

        for p in task.solve.into_iter() {
            solve.push(
                p.file_stem()
                    .expect(&format!("problem {:?} has no name", p))
                    .to_string_lossy()
                    .to_string(),
            );
        }

        tasks.push(Task { name, learn, solve })
    }
    Ok(Instance {
        learn_dir,
        solve_dir,
        learn_mem_limit: suite.memory_limit_learn,
        solve_mem_limit: suite.memory_limit_solve,
        runners,
        tasks,
        attributes,
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
    fs::create_dir_all(&dir)?;
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
        exe.to_string_lossy(),
        args.iter()
            .map(|arg| format!(" {}", arg))
            .collect::<String>()
    ));
    content.push_str(&format!("$(eval \"{}\"&>log)\n", command));
    content.push_str(&format!("echo $? > exit_code"));
    let runner_path = dir.join("runner.sh");
    fs::write(&runner_path, content)?;
    let mut cmd = Command::new("chmod");
    cmd.arg("u+x");
    cmd.arg(&runner_path);
    cmd.status()?;
    Ok(runner_path)
}
