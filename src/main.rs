mod misc;
mod runner;
mod setup;

use crate::{misc::logging, setup::setup};
use clap::Parser;
use itertools::Itertools;
use log::trace;
use path_absolutize::*;
use runner::RunnerKind;
use setup::suite::generate_suite;
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Specifies the directory where instance directories are generated
    #[arg(short, long, required = false, default_value = "/tmp/labyr")]
    instance_dir: PathBuf,

    /// Specifies which directory results will be written to
    #[arg(short, long, required = false, default_value = "results.csv")]
    out: PathBuf,

    /// Whether to keep the instance dir, deleted by default
    #[arg(short, long, required = false)]
    keep_instance_dir: bool,

    /// The maximum number of threads to use for local runner, 0 for max
    #[arg(short, long, required = false, default_value_t = 1)]
    threads: usize,

    #[arg(short, long, required = false, default_value = "local")]
    runner: RunnerKind,

    /// The suite to run
    #[arg(required = true)]
    suite: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    logging::init();
    trace!("Parsing args");
    let args = Args::parse();
    let result = _main(&args);
    if !args.keep_instance_dir {
        trace!("Deleting instance dir");
        let _ = fs::remove_dir_all(args.instance_dir);
    }
    result
}

fn _main(args: &Args) -> Result<(), Box<dyn Error>> {
    trace!("Cleaning working dir");
    let working_dir = args.instance_dir.to_owned().absolutize()?.to_path_buf();
    let out_file = args.out.to_owned().absolutize()?.to_path_buf();
    trace!("Load data");
    let suite = generate_suite(&args.suite)?;
    trace!("Setting up");
    let instances = setup(&suite, &working_dir, args.threads)?;
    trace!("Generating runner");
    let runner = runner::generate(&args);
    trace!("Running learners");
    runner.run(&instances.learners);
    trace!("Running solvers");
    let results = runner.run(&instances.solvers);
    let attributes: Vec<String> = results
        .iter()
        .flat_map(|a| {
            if let Some(attributes) = instances.solvers[a.id].attributes {
                return attributes
                    .patterns
                    .iter()
                    .map(|a| a.name.to_owned())
                    .collect::<Vec<String>>();
            }
            return Vec::<String>::new();
        })
        .unique()
        .collect();
    let header = format!(
        "id,domain,problem,solver,exit_code,solved,execution_time{}",
        attributes
            .iter()
            .map(|a| format!(",{}", a))
            .collect::<String>()
    );
    let mut file = File::create(out_file)?;
    write!(file, "{}\n", header)?;
    for result in results.iter() {
        let solver = &instances.solvers[result.id];
        let row = format!(
            "{},{},{},{},{},{},{}{}",
            result.id,
            solver.domain,
            match solver.kind {
                setup::generation::InstanceKind::Learner => panic!(),
                setup::generation::InstanceKind::Solver { problem } => problem,
            },
            solver.name,
            result.exit_status.code().unwrap(),
            result.exit_status.success(),
            result.time.as_secs_f64(),
            attributes
                .iter()
                .map(|a| format!(
                    ",{}",
                    match result.attributes.contains_key(a) {
                        true => result.attributes.get(a).unwrap(),
                        false => "",
                    }
                ))
                .collect::<String>()
        );
        write!(file, "{}\n", row)?;
    }
    Ok(())
}
