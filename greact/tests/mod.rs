mod r#async;
mod sync;

use std::time::{Duration, Instant};

pub struct Run {
    stop_at: Instant,
}
impl Run {
    pub fn run_for() -> Duration {
        Duration::from_secs(3)
    }
    pub fn new() -> Self {
        let stop_at = Instant::now() + Self::run_for();
        Self { stop_at }
    }
    pub fn should_stop(&self) -> bool {
        Instant::now() >= self.stop_at
    }

    pub fn run(self, mut work: impl FnMut() -> Result<(), String>) -> Result<(), String> {
        loop {
            work()?;
            if self.should_stop() {
                return Ok(());
            }
        }
    }
    pub fn launch(work: impl FnMut() -> Result<(), String>) {
        match Self::new().run(work) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("something went wrong: {e}");
                panic!("an error occurred")
            }
        }
    }
}

pub fn run_test(work: impl FnMut() -> Result<(), String>) {
    Run::launch(work)
}
