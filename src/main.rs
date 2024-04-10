mod evaluation;
mod execution;
mod misc;
mod setup;

use crate::misc::logging;
use clap::Parser;
use execution::ExecutionKind;
use log::{info, trace};
use path_absolutize::Absolutize;
use std::{fs, path::PathBuf, thread::available_parallelism};
use tempfile::tempdir_in;

pub type Result<T> = std::result::Result<T, String>;

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

    /// The suite to run
    #[arg(required = true)]
    suite: PathBuf,
}

fn main() -> Result<()> {
    logging::init();
    trace!("Reading args");
    let args = Args::parse();
    let out_dir = args
        .out
        .absolutize()
        .map_err(|e| {
            format!(
                "Failed to absolutize out dir {:?} with error: {}",
                args.out, e
            )
        })?
        .to_path_buf();
    trace!("Creating work dir");
    fs::create_dir_all(&args.work_dir)
        .map_err(|e| format!("Failed to create work dir with error: {}", e))?;
    let temp_dir: tempfile::TempDir = tempdir_in(&args.work_dir)
        .map_err(|e| format!("Failed to create temp dir with error: {}", e))?;
    let result = _main(&args, &temp_dir.path().to_path_buf(), &out_dir);
    if args.keep_working_dir {
        trace!("Releasing temp dir");
        let _ = temp_dir.into_path();
    }
    result
}

fn _main(args: &Args, temp_dir: &PathBuf, out_dir: &PathBuf) -> Result<()> {
    trace!("Determining number of threads");
    let threads = match args.threads {
        0 => available_parallelism()
            .map_err(|e| format!("Failed to get available threads: {}", e))?
            .get(),
        _ => args.threads,
    };
    info!("Thread count: {}", threads);
    let suite_path = args
        .suite
        .absolutize()
        .map_err(|e| format!("Failed to get absolute path of suite: {}", e))?
        .to_path_buf();
    trace!("Generating instance");
    let instance = setup::run(&temp_dir, &suite_path)?;
    trace!("Executing instance");
    execution::execute(instance.to_owned(), args.execution_kind, threads)?;
    evaluation::eval(&out_dir, &instance)?;
    Ok(())
}
