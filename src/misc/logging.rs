use env_logger::{Env, Logger};
use indicatif::{MultiProgress, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use once_cell::sync::OnceCell;

static LOGGER: OnceCell<Logger> = OnceCell::new();
static PROGRESSER: OnceCell<MultiProgress> = OnceCell::new();

fn logger() -> &'static Logger {
    LOGGER.get().expect("logging is not initialized")
}

fn progresser() -> &'static MultiProgress {
    PROGRESSER.get().expect("logging is not initialized")
}

pub fn init() {
    let _ =
        LOGGER.set(env_logger::Builder::from_env(Env::default().default_filter_or("trace")).build());
    let _ = PROGRESSER.set(MultiProgress::new());
    LogWrapper::new(progresser().clone(), logger())
        .try_init()
        .unwrap()
}

pub struct ProgressBar {
    pg: indicatif::ProgressBar,
}

impl ProgressBar {
    pub fn new(len: usize) -> Self {
        let pg = indicatif::ProgressBar::new(len as u64);
        pg.set_style(
            ProgressStyle::with_template("[{bar:32}] {pos}/{len}")
                .unwrap()
                .progress_chars("=> "),
        );
        let pg = progresser().add(pg);
        Self { pg }
    }
    pub fn inc(&self) {
        self.pg.inc(1);
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        self.pg.finish();
        progresser().remove(&self.pg);
    }
}
