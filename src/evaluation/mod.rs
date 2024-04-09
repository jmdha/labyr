mod csv;
mod learn;
mod solve;

use crate::setup::instance::Instance;
use crate::Result;
use std::fs;
use std::path::PathBuf;

pub fn eval(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Failed to create output dir with error: {}", e))?;
    csv::collect(out_dir, instance)?;
    learn::collect(out_dir, instance)?;
    solve::collect(out_dir, instance)?;
    Ok(())
}
