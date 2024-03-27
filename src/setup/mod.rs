pub mod generation;
pub mod suite;

use self::generation::generate_instances;
use self::generation::Instances;
use self::suite::Suite;
use log::trace;
use std::env;
use std::error::Error;
use std::path::PathBuf;

pub fn setup<'a>(
    suite: &'a Suite,
    working_dir: &'a PathBuf,
    threads: usize,
) -> Result<Instances<'a>, Box<dyn Error>> {
    if let Some(mem) = suite.memory_limit_learn {
        trace!("Limiting c# mem use");
        env::set_var("DOTNET_GCHeapHardLimit", format!("{}", mem * 1000));
    }
    trace!("Building thread pool");
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
    trace!("Generating instances");
    let working_dir = match working_dir.is_relative() {
        true => working_dir.canonicalize()?,
        false => working_dir.to_path_buf(),
    };
    generate_instances(&working_dir, &suite)
}
