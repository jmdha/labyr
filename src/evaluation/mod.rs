mod csvs;

use self::csvs::collect_csvs;
use crate::setup::instance::Instance;
use crate::Result;
use std::path::PathBuf;

pub fn eval(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    collect_csvs(out_dir, instance)?;
    Ok(())
}
