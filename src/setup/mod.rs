pub mod generation;
pub mod suite;

use self::generation::generate_instances;
use self::generation::Instance;
use self::suite::Suite;
use log::trace;
use std::path::PathBuf;

pub fn setup<'a>(
    suite: &'a Suite,
    working_dir: &'a PathBuf,
    threads: usize,
) -> Result<Vec<Instance<'a>>, Box<dyn std::error::Error>> {
    trace!("Building thread pool");
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
    trace!("Generating instances");
    generate_instances(suite.memory_limit, suite.time_limit, working_dir, &suite)
}
