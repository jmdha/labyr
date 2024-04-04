use crate::misc::logging::ProgressBar;
use crate::setup::instance::Instance;
use crate::Result;
use log::{info, trace};
use pretty_duration::pretty_duration;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

pub fn execute(instance: &Instance) -> Result<()> {
    trace!("Running learners");
    _execute(&instance.learners.iter().map(|l| (&l.dir, &l.exe)).collect())?;
    trace!("Running solvers");
    _execute(&instance.solvers.iter().map(|l| (&l.dir, &l.exe)).collect())?;
    Ok(())
}

fn _execute(scripts: &Vec<(&PathBuf, &PathBuf)>) -> Result<()> {
    let width = ((scripts.len() as f64).log10()).ceil() as usize;
    let pg = ProgressBar::new(scripts.len());
    scripts.into_par_iter().for_each(|(dir, exe)| {
        let dir_name = dir.file_stem().expect("Could not retrieve name of dir");
        let mut command = Command::new(exe);
        command.current_dir(dir);
        trace!("Running command: {:?}", command);
        let t = Instant::now();
        let output = command.output().expect("Failed to run command");
        let elapsed = t.elapsed();
        info!(
            "{:0>width$} {} - {}",
            dir_name.to_str().expect("Could not convert name to string"),
            output.status,
            pretty_duration(&elapsed, None),
            width = width,
        );
        pg.inc();
    });
    Ok(())
}
