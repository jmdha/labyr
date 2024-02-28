mod misc;
mod runner;
mod setup;

use crate::misc::logging;
use clap::Parser;
use itertools::Itertools;
use log::trace;
use runner::RunnerKind;
use setup::setup;
use setup::suite::generate_suite;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init();
    let args = Args::parse();
    trace!("Load data");
    let suite = generate_suite(&args.suite)?;
    trace!("Generate setup");
    let instances: Vec<setup::generation::Instance<'_>> =
        setup(&suite, &args.working_dir, args.threads)?;
    let runner = runner::generate(&args);
    let results = runner.run(instances);
    let attributes: Vec<String> = results
        .iter()
        .flat_map(|a| {
            a.attributes
                .iter()
                .map(|a| a.0.to_owned().to_owned())
                .collect::<Vec<String>>()
        })
        .unique()
        .collect();
    let header = format!(
        "id,domain,problem,solver,exit_code,execution_time{}",
        attributes
            .iter()
            .map(|a| format!(",{}", a))
            .collect::<String>()
    );
    let mut file = File::create(args.out)?;
    write!(file, "{}\n", header)?;
    for result in results.iter() {
        let row = format!(
            "{},{},{},{},{},{}{}",
            result.id,
            result.domain,
            result.problem,
            result.solver,
            result.exit_status.code().unwrap(),
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
