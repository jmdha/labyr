mod evaluation;
mod execution;
mod misc;
mod setup;

use crate::misc::logging;
use anyhow::Result;
use clap::Parser;
use execution::ExecutionKind;
use log::{info, trace};
use path_absolutize::Absolutize;
use std::{fs, path::PathBuf, thread::available_parallelism};
use tempfile::tempdir_in;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Specifies directory wherein work dir will be created
    #[arg(short, long, required = false, default_value = "/tmp")]
    work_dir: PathBuf,

    /// Specifies which directory results will be written to
    #[arg(short, long, required = false, default_value = "results")]
    out: PathBuf,

    /// Whether to keep working dir
    #[arg(short, long, required = false)]
    keep_working_dir: bool,

    /// The maximum number of threads to use for local runner, 0 for max
    #[arg(short, long, required = false, default_value_t = 1)]
    threads: usize,

    #[arg(short, long, required = false, default_value = "local")]
    execution_kind: ExecutionKind,

    /// Continues a prior run, redoing those that have no exit code
    #[arg(long)]
    prior_run: Option<PathBuf>,

    /// Given "prior_run" forces redoing of learning regardless of prior state
    #[arg(long, default_value = "false")]
    force_learn: bool,

    /// Given "prior_run" forces redoing of solving regardless of prior state.
    /// Is implicit if "force_learn" is given
    #[arg(long, default_value = "false")]
    force_solve: bool,

    /// The suite to run
    #[arg(required = true)]
    suite: PathBuf,
}

fn main() -> Result<()> {
    logging::init();
    trace!("Reading args");
    let args = Args::parse();
    let out_dir = args.out.absolutize()?.to_path_buf();
    match &args.prior_run {
        Some(path) => _main(&args, &path, &out_dir),
        None => {
            trace!("Creating work dir");
            fs::create_dir_all(&args.work_dir)?;
            let temp_dir: tempfile::TempDir = tempdir_in(&args.work_dir)?;
            let result = _main(&args, &temp_dir.path().to_path_buf(), &out_dir);
            if args.keep_working_dir {
                trace!("Releasing temp dir");
                let _ = temp_dir.into_path();
            }
            result
        }
    }
}

fn _main(args: &Args, temp_dir: &PathBuf, out_dir: &PathBuf) -> Result<()> {
    trace!("Determining number of threads");
    let threads = match args.threads {
        0 => available_parallelism()?.get(),
        _ => args.threads,
    };
    info!("Thread count: {}", threads);
    let suite_path = args.suite.absolutize()?.to_path_buf();
    trace!("Generating instance");
    let instance = setup::run(&temp_dir, &suite_path, args.force_learn, args.force_solve)?;
    trace!("Executing instance");
    execution::execute(instance.to_owned(), args.execution_kind, threads)?;
    evaluation::eval(&out_dir, &instance)?;
    Ok(())
}
