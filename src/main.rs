mod execution;
mod instance;
mod misc;
mod register;
mod suite;

use anyhow::Result;
use clap::Parser;
use execution::ExecutionKind;
use path_absolutize::Absolutize;
use std::{fs, path::PathBuf, thread::available_parallelism};
use tempfile::tempdir_in;

use crate::register::Register;

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
    println!("reading args...");
    let args = Args::parse();
    println!("finding out dir...");
    let out_dir = args.out.absolutize()?.to_path_buf();
    println!("creating work dir...");
    fs::create_dir_all(&args.work_dir)?;
    println!("creating temp dir...");
    let temp_dir: tempfile::TempDir = tempdir_in(&args.work_dir)?;
    let result = _main(
        &args,
        &temp_dir.path().to_path_buf(),
        &out_dir,
        args.keep_working_dir,
    );
    if args.keep_working_dir {
        println!("releasing temp dir...");
        let _ = temp_dir.into_path();
    }
    result
}

fn _main(args: &Args, temp_dir: &PathBuf, out_dir: &PathBuf, keep_dirs: bool) -> Result<()> {
    println!("determining number of threads...");
    let threads = match args.threads {
        0 => available_parallelism()?.get(),
        _ => args.threads,
    };
    println!("thread count: {}", threads);
    println!("building thread pool...");
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
    println!("finding suite...");
    let suite_path = args.suite.absolutize()?.to_path_buf();
    println!("loading suite...");
    suite::load(&suite_path)?;
    println!("generating instances...");
    let instances = instance::generate(
        &suite_path.parent().expect("suite in no dir?").to_path_buf(),
        suite::get_suite(),
        temp_dir,
    )?;
    println!("executing instances...");
    let register = {
        let mut register = Register::default();
        execution::execute(&mut register, instances, args.execution_kind)?;
        register
    };
    println!("exporting to {:?}...", out_dir);
    register.export(out_dir)?;
    Ok(())
}
