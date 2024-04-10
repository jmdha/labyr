use crate::setup::instance::{Instance, RunKind};
use crate::Result;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn execute(instance: Instance, _: usize) -> Result<()> {
    {
        let executer = generate_executer(&instance.learn_dir)?;
        let _ = execute_learn(&instance, &executer.path().to_path_buf());
    }
    {
        let executer = generate_executer(&instance.solve_dir)?;
        let _ = execute_solve(&instance, &executer.path().to_path_buf());
    }
    Ok(())
}

fn execute_learn(instance: &Instance, executer: &PathBuf) -> Result<()> {
    let array = format!(
        "--array=0-{}",
        instance
            .runs
            .iter()
            .filter(|r| r.kind == RunKind::Learner)
            .count()
    );
    let _ = Command::new("sbatch")
        .args([
            "--wait",
            &array,
            "--job-name=P10_Meta_Learn",
            &executer.to_string_lossy(),
        ])
        .output();
    Ok(())
}

fn execute_solve(instance: &Instance, executer: &PathBuf) -> Result<()> {
    let array = format!(
        "--array=0-{}",
        instance
            .runs
            .iter()
            .filter(|r| r.kind != RunKind::Learner)
            .count()
    );
    let _ = Command::new("sbatch")
        .args([
            "--wait",
            &array,
            "--job-name=P10_Meta_Solve",
            &executer.to_string_lossy(),
        ])
        .output();
    Ok(())
}

fn generate_executer(dir: &PathBuf) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new_in(dir).map_err(|e| {
        format!(
            "Failed to generate slurm executer temp file with error: {}",
            e
        )
    })?;

    let _ = writeln!(file, "#!/bin/bash\n");
    let _ = writeln!(file, "#SBATCH --mem=16G\n");
    let _ = writeln!(
        file,
        "DIR={}/${{SLURM_ARRAY_TASK_ID}}\n",
        dir.to_string_lossy()
    );
    let _ = writeln!(file, "cd ${{DIR}}\n");
    let _ = writeln!(file, "./runner.sh\n");

    Ok(file)
}
