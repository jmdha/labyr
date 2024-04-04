pub mod instance;
mod suite;

use crate::{setup::instance::Instance, Result};
use log::trace;
use std::{env, fs, path::PathBuf};

pub fn run(temp_dir: &PathBuf, suite_path: &PathBuf) -> Result<Instance> {
    trace!("Reading suite file");
    let suite_content =
        fs::read_to_string(suite_path).map_err(|e| format!("Failed to read suite file: {}", e))?;
    trace!("Changing working directory to {:?} parent", suite_path);
    env::set_current_dir(suite_path.parent().ok_or("Suite files has no parent")?)
        .map_err(|e| format!("Failed to change working directory: {}", e))?;
    trace!("Parsing suite file");
    let suite = suite::parse(&suite_content)?;
    trace!("Changing working directory to {:?}", temp_dir);
    env::set_current_dir(temp_dir)
        .map_err(|e| format!("Failed to change working directory: {}", e))?;
    trace!("Generating instance");
    instance::generate(suite)
}
