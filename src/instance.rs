use crate::suite::{Attribute, Suite, Task};
use anyhow::Result;
use itertools::Itertools;
use std::{fs, path::PathBuf, process::Command};

#[derive(Debug, Clone)]
pub struct Instance {
    pub dir: PathBuf,
    pub learn: Option<Learn>,
    pub solve: Option<Vec<Solve>>,
}

#[derive(Debug, Clone)]
pub struct Learn {
    pub task: &'static Task,
    pub args: Vec<String>,
    pub dir: PathBuf,
    pub path: PathBuf,
    pub attributes: &'static Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct Solve {
    pub task: &'static Task,
    pub args: Vec<String>,
    pub dir: PathBuf,
    pub path: PathBuf,
    pub problem: &'static String,
    pub attributes: &'static Vec<Attribute>,
}

pub fn generate(
    suite_dir: &PathBuf,
    suite: &'static Suite,
    dir: &PathBuf,
) -> Result<Vec<Instance>> {
    let (var_learn_args, learn_args): (Vec<usize>, Option<Vec<Vec<&String>>>) = match &suite.learn {
        Some(learn) => match &learn.args {
            Some(args) => (
                args.iter()
                    .enumerate()
                    .filter(|(_, args)| args.len() > 1)
                    .map(|(i, _)| i)
                    .collect(),
                Some(args.iter().multi_cartesian_product().collect()),
            ),
            None => (vec![], Some(vec![vec![]])),
        },
        None => (vec![], None),
    };
    let (var_solve_args, solve_args): (Vec<usize>, Option<Vec<Vec<&String>>>) = match &suite.solve {
        Some(solve) => match &solve.args {
            Some(args) => (
                args.iter()
                    .enumerate()
                    .filter(|(_, args)| args.len() > 1)
                    .map(|(i, _)| i)
                    .collect(),
                Some(args.iter().multi_cartesian_product().collect()),
            ),
            None => (vec![], Some(vec![vec![]])),
        },
        None => (vec![], None),
    };
    let instance_args: Vec<(Option<Vec<&String>>, Option<Vec<Vec<&String>>>)>;
    if let Some(learn_args) = learn_args {
        instance_args = learn_args
            .iter()
            .map(|l_args| (Some(l_args.to_owned()), solve_args.to_owned()))
            .collect();
    } else if let Some(solve_args) = solve_args {
        instance_args = vec![(None, Some(solve_args))];
    } else {
        instance_args = vec![];
    }

    let mut instances = vec![];
    for task in suite.tasks.iter() {
        let dir = dir.join(&task.name);
        for (i, args) in instance_args.iter().enumerate() {
            let dir = dir.join(format!("{}", i));
            let learn = match &args.0 {
                Some(args) => Some(generate_learn(
                    suite_dir,
                    suite,
                    task,
                    &dir.join("learn"),
                    &args,
                    &var_learn_args,
                )?),
                None => None,
            };
            let solve = match &args.1 {
                Some(args) => Some(generate_solve(
                    suite_dir,
                    suite,
                    match &learn {
                        Some(learn) => Some(&learn.dir),
                        None => None,
                    },
                    task,
                    &dir,
                    &args,
                    &var_solve_args,
                )?),
                None => None,
            };
            instances.push(Instance { dir, learn, solve });
        }
    }
    Ok(instances)
}

fn generate_learn(
    suite_dir: &PathBuf,
    suite: &'static Suite,
    task: &'static Task,
    dir: &PathBuf,
    args: &Vec<&String>,
    var_args: &Vec<usize>,
) -> Result<Learn> {
    let args: Vec<String> = args
        .iter()
        .map(|arg| {
            let arg = arg.replace("DIR", &suite_dir.to_string_lossy());
            let arg = arg.replace("TASK", &task.name);
            let arg = arg.replace("DOMAIN", &task.domain.to_string_lossy());
            let arg = arg.replace(
                "PROBLEMS",
                &task
                    .learn
                    .iter()
                    .map(|(_, p)| p.to_string_lossy())
                    .join(" "),
            );
            arg
        })
        .collect();
    let var_args = var_args.iter().map(|i| args[*i].to_owned()).collect();
    let path = generate_script(
        &dir,
        &suite.learn.as_ref().unwrap().path,
        &args,
        suite.learn.as_ref().unwrap().time_limit,
        suite.learn.as_ref().unwrap().memory_limit,
    )?;
    Ok(Learn {
        task,
        args: var_args,
        dir: dir.to_owned(),
        path,
        attributes: &suite.learn.as_ref().unwrap().attributes,
    })
}

fn generate_solve(
    suite_dir: &PathBuf,
    suite: &'static Suite,
    learn_dir: Option<&PathBuf>,
    task: &'static Task,
    dir: &PathBuf,
    all_args: &Vec<Vec<&String>>,
    var_args: &Vec<usize>,
) -> Result<Vec<Solve>> {
    let mut exes = vec![];
    for problem in task.solve.iter() {
        let dir = dir.join(&problem.0);
        for (i, args) in all_args.into_iter().enumerate() {
            let dir = dir.join(format!("{}", i));
            let args: Vec<String> = args
                .iter()
                .map(|arg| {
                    let arg = arg.replace("TASK", &task.name);
                    let arg = arg.replace("DOMAIN", &task.domain.to_string_lossy());
                    let arg = arg.replace("PROBLEM", &problem.1.to_string_lossy());
                    let arg = match arg.contains("LEARN_DIR") {
                        true => arg.replace(
                            "LEARN_DIR",
                            &learn_dir
                                .as_ref()
                                .expect("learn dir required in solve, but no learner defined")
                                .to_string_lossy(),
                        ),
                        false => arg,
                    };
                    let arg = arg.replace("DIR", &suite_dir.to_string_lossy());
                    arg
                })
                .collect();
            let path = generate_script(
                &dir,
                &suite.solve.as_ref().unwrap().path,
                &args,
                suite.solve.as_ref().unwrap().time_limit,
                suite.solve.as_ref().unwrap().memory_limit,
            )?;
            let var_args = var_args.iter().map(|i| args[*i].to_owned()).collect();
            exes.push(Solve {
                task,
                dir,
                args: var_args,
                path,
                problem: &problem.0,
                attributes: &suite.solve.as_ref().unwrap().attributes,
            });
        }
    }
    Ok(exes)
}

fn generate_script(
    dir: &PathBuf,
    exe: &PathBuf,
    args: &Vec<String>,
    time_limit: Option<usize>,
    memory_limit: Option<usize>,
) -> Result<PathBuf> {
    println!("creating dir {:?}...", dir);
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
        "{}{}",
        exe.to_string_lossy(),
        args.iter()
            .map(|arg| format!(" {}", arg))
            .collect::<String>()
    ));
    content.push_str(&format!("$(eval \"{}\">log)\n", command));
    content.push_str(&format!("echo $? > exit_code"));
    let runner_path = dir.join("runner.sh");
    println!("generating runner {:?}...", runner_path);
    fs::write(&runner_path, content)?;
    println!("granting rights to runner...");
    let mut cmd = Command::new("chmod");
    cmd.arg("u+x");
    cmd.arg(&runner_path);
    cmd.status()?;
    Ok(runner_path)
}
