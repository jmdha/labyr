use itertools::Itertools;
use log::trace;

use crate::{setup::instance::Instance, Result};
use std::{collections::HashSet, fs, path::PathBuf};

pub fn eval(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    collect_csvs(out_dir, instance)?;
    Ok(())
}

fn collect_csvs(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    let csvs: HashSet<PathBuf> = instance
        .learners
        .iter()
        .flat_map(|l| find_files(&l.dir, ".csv"))
        .collect();

    for csv in csvs.iter() {
        let mut content: Vec<String> = Default::default();
        for run in instance.learners.iter() {
            let csv_loc = run.dir.join(csv);
            if !csv_loc.exists() {
                continue;
            }
            trace!("Reading csv: {:?}", csv_loc);
            let csv_content = fs::read_to_string(&csv_loc)
                .map_err(|e| format!("Failed to read csv {:?} with error: {}", csv_loc, e))?;
            let lines: Vec<String> = csv_content.lines().map(|l| l.to_string()).collect();
            if lines.is_empty() {
                continue;
            }
            if content.is_empty() {
                content.push(lines[0].to_owned());
            }
            for line in lines.into_iter().skip(1) {
                content.push(line);
            }
        }
        let csv_out = out_dir.join(csv);
        if let Some(dir) = csv_out.parent() {
            fs::create_dir_all(dir).map_err(|e| {
                format!(
                    "Failed to create csv dirs for csv {:?} with error: {}",
                    csv_out, e
                )
            })?;
        }
        fs::write(&csv_out, content.into_iter().join("\n"))
            .map_err(|e| format!("Failed to write csv {:?} with error: {}", csv_out, e))?;
    }
    Ok(())
}

/// Recursively finds all files with extension
fn find_files(dir: &PathBuf, ext: &str) -> Vec<PathBuf> {
    let mut files = vec![];
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy();
        if name.ends_with(ext) {
            files.push(entry.path().strip_prefix(dir).unwrap().to_path_buf());
        }
    }
    files
}