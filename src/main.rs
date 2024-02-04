mod misc;
mod setup;

use clap::Parser;
use env_logger::Env;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use log::{info, trace};
use pretty_duration::pretty_duration;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use setup::setup;
use setup::suite::generate_suite;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::setup::suite::Attributes;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, required = false, default_value = "/tmp/labyr")]
    working_dir: PathBuf,

    #[arg(short, long, required = false, default_value = "./")]
    result_dir: PathBuf,

    #[arg(short, long, required = false, default_value_t = 1)]
    threads: usize,

    /// By default the program waits 3s after setup, in case user wants to cancel.
    /// If this flag is given, this wait is skipped
    #[arg(short, long)]
    skip_delay: bool,

    #[arg(required = true)]
    suite: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = env_logger::Builder::from_env(Env::default().default_filter_or("info")).build();
    let progresser = MultiProgress::new();
    LogWrapper::new(progresser.clone(), logger)
        .try_init()
        .unwrap();
    let args = Args::parse();
    trace!("Load data");
    let suite = generate_suite(&args.suite)?;
    trace!("Generate setup");
    let instances = setup(&suite, &args.working_dir, args.threads)?;
    let width = ((instances.len() as f64).log10()).ceil() as usize;

    // In case something happended in setup
    if !args.skip_delay {
        info!("If any warnings given, consider stopping program. Program will continue in 3s");
        sleep(Duration::from_secs(3));
    }

    let pg = ProgressBar::new(instances.len() as u64);
    pg.set_style(
        ProgressStyle::with_template("[{bar:32}] {pos}/{len}")
            .unwrap()
            .progress_chars("=> "),
    );
    let pg = progresser.add(pg);
    let commands = instances.par_iter().map(|instance| {
        let mut command = Command::new(&instance.exe);
        command.current_dir(&instance.dir);
        let t = Instant::now();
        let output = command.output().expect("failed to spawn command");
        let execution_time = t.elapsed();
        let status = output.status;
        info!(
            "{:0>width$} {} - {}",
            instance.dir.file_stem().unwrap().to_str().unwrap(),
            status,
            pretty_duration(&execution_time, None),
            width = width,
        );
        let mut log_file = File::create(Path::new(&instance.dir).join("log")).unwrap();
        let output = str::from_utf8(&output.stdout).unwrap().to_owned();
        let _ = write!(log_file, "{}", output);
        pg.inc(1);
        (instance, output, status, execution_time)
    });
    let output: Vec<_> = commands.collect();
    pg.finish();
    progresser.remove(&pg);

    let mut results: HashMap<String, File> = HashMap::new();
    for attributes in suite.attributes.iter() {
        let mut file = File::create(args.result_dir.join(&attributes.name))?;
        let header = format!(
            "id,domain,problem,solver,exit_code,execution_time{}",
            attributes
                .patterns
                .iter()
                .map(|p| format!(",{}", &p.name))
                .collect::<String>()
        );
        write!(file, "{}\n", header)?;
        results.insert(attributes.name.to_owned(), file);
    }

    for (instance, output, status, execution_time) in output.into_iter() {
        let attributes: &Attributes = match instance.attributes {
            Some(attributes) => attributes,
            None => continue,
        };
        let file: &mut File = results.get_mut(&attributes.name).unwrap();
        let row = format!(
            "{},{},{},{},{},{}{}\n",
            instance.id,
            instance.domain,
            instance.problem,
            instance.solver,
            status.code().unwrap(),
            execution_time.as_millis(),
            attributes
                .patterns
                .iter()
                .map(|pattern| format!(
                    ",{}",
                    match pattern.pattern.captures(&output) {
                        Some(c) => c[1].to_string(),
                        None => "".to_string(),
                    }
                ))
                .collect::<String>()
        );
        write!(file, "{}", row)?;
    }

    Ok(())
}
