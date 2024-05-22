use crate::setup::instance::{Instance, RunKind};
use anyhow::Result;
use log::info;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::NamedTempFile;

pub fn execute(instance: Instance) -> Result<()> {
    if instance
        .runs
        .iter()
        .any(|r| r.kind == RunKind::Learner && !r.skip)
    {
        info!("Running learn");
        let executer = generate_executer(&instance.learn_dir, instance.learn_mem_limit)?;
        println!(
            "{:?}",
            execute_learn(&instance, &executer.path().to_path_buf())
        );
    }
    if instance
        .runs
        .iter()
        .any(|r| r.kind != RunKind::Learner && !r.skip)
    {
        info!("Running solve");
        let executer = generate_executer(&instance.solve_dir, instance.solve_mem_limit)?;
        println!(
            "{:?}",
            execute_solve(&instance, &executer.path().to_path_buf())
        );
    }
    Ok(())
}

fn execute_learn(instance: &Instance, executer: &PathBuf) -> Result<Output> {
    let array = format!(
        "--array=0-{}",
        instance
            .runs
            .iter()
            .filter(|r| r.kind == RunKind::Learner)
            .count()
            - 1
    );
    Ok(Command::new("sbatch")
        .args([
            "--wait",
            &array,
            "--job-name=P10_Meta_Learn",
            &executer.to_string_lossy(),
        ])
        .output()?)
}

fn execute_solve(instance: &Instance, executer: &PathBuf) -> Result<Output> {
    let array = format!(
        "--array=0-{}",
        instance
            .runs
            .iter()
            .filter(|r| r.kind != RunKind::Learner)
            .count()
            - 1
    );
    Ok(Command::new("sbatch")
        .args([
            "--wait",
            &array,
            "--job-name=P10_Meta_Solve",
            &executer.to_string_lossy(),
        ])
        .output()?)
}

fn generate_executer(dir: &PathBuf, mem_limit: Option<usize>) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new_in(dir)?;

    let _ = writeln!(file, "#!/bin/bash\n");
    let _ = writeln!(
        file,
        "#SBATCH --mem={}G\n",
        match mem_limit {
            Some(lim) => lim.div_ceil(999),
            None => 16,
        }
    );
    let _ = writeln!(
        file,
        "DIR={}/${{SLURM_ARRAY_TASK_ID}}\n",
        dir.to_string_lossy()
    );
    let _ = writeln!(file, "cd ${{DIR}}\n");
    let _ = writeln!(file, "./runner.sh\n");

    Ok(file)
}
