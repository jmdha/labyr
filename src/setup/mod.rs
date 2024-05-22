pub mod instance;
pub mod suite;

use crate::setup::instance::Instance;
use anyhow::Result;
use log::trace;
use std::{env, fs, path::PathBuf};

pub fn run(
    temp_dir: &PathBuf,
    suite_path: &PathBuf,
    force_learn: bool,
    force_solve: bool,
) -> Result<Instance> {
    trace!("Reading suite file");
    let suite_content = fs::read_to_string(suite_path)?;
    trace!("Changing working directory to {:?} parent", suite_path);
    env::set_current_dir(suite_path.parent().expect("suite is an orphan"))?;
    trace!("Parsing suite file");
    let suite = suite::parse(&suite_content)?;
    trace!("Changing working directory to {:?}", temp_dir);
    env::set_current_dir(temp_dir)?;
    trace!("Generating instance");
    instance::generate(suite, force_learn, force_solve)
}
