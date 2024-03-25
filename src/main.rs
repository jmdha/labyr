mod misc;
mod runner;
mod setup;

use crate::{misc::logging, setup::setup};
use clap::Parser;
use log::trace;
use runner::RunnerKind;
use setup::suite::generate_suite;
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, required = false, default_value = "/tmp/labyr")]
    working_dir: PathBuf,

    #[arg(short, long, required = false, default_value = "results.csv")]
    out: PathBuf,

    #[arg(short, long, required = false, default_value_t = 1)]
    threads: usize,

    #[arg(short, long, required = false, default_value = "local")]
    runner: RunnerKind,

    #[arg(required = true)]
    suite: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    logging::init();
    let args = Args::parse();
    trace!("Load data");
    let suite = generate_suite(&args.suite)?;
    trace!("Setting up");
    let instances = setup(&suite, &args.working_dir, args.threads)?;
    trace!("Generating runner");
    let runner = runner::generate(&args);
    trace!("Running learners");
    runner.run(instances.learners);
    trace!("Running solvers");
    runner.run(instances.solvers);
    Ok(())
}
