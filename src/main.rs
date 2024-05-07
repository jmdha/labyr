mod misc;
mod suite;

use anyhow::Result;
use clap::Parser;
use path_absolutize::Absolutize;
use std::{fs, path::PathBuf};
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

    /// The suite to run
    #[arg(required = true)]
    suite: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match fs::create_dir_all(&args.work_dir) {
        Ok(_) => {}
        Err(e) => panic!("Could not create work dir: {}", e),
    };
    let temp_dir: tempfile::TempDir = match tempdir_in(&args.work_dir) {
        Ok(dir) => dir,
        Err(e) => panic!("Could not create temp dir in work dir: {}", e),
    };
    let result = _main(&args, &temp_dir.path().to_path_buf());
    if args.keep_working_dir {
        let _ = temp_dir.into_path();
    }
    result
}

fn _main(args: &Args, temp_dir: &PathBuf) -> Result<()> {
    let suite_path = match args.suite.absolutize() {
        Ok(path) => path.to_path_buf(),
        Err(e) => panic!("Could not absolutize suite path: {}", e),
    };
    let suite = suite::load(&suite_path)?;
    Ok(())
}
