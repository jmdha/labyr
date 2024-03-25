use super::{Result, Runner};
use crate::{misc::logging::ProgressBar, setup::generation::Instance};
use log::info;
use pretty_duration::pretty_duration;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::io::Write;
use std::{fs::File, path::Path, process::Command, time::Instant};

pub struct Local {}

impl Runner for Local {
    fn run<'a>(&'a self, instances: Vec<Instance<'a>>) -> Vec<Result> {
        let width = ((instances.len() as f64).log10()).ceil() as usize;
        let pg = ProgressBar::new(instances.len());
        let commands = instances.into_par_iter().map(|instance| {
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
            let output = std::str::from_utf8(&output.stdout).unwrap().to_owned();
            let _ = write!(log_file, "{}", output);
            pg.inc();
            (instance, output, status, execution_time)
        });
        commands
            .enumerate()
            .map(|(i, a)| Result {
                id: i,
                exit_status: a.2,
                time: a.3,
            })
            .collect()
    }
}
