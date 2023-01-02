use log::{log_enabled, Level};
pub use timer::Timer;

const TIMER_LEVEL: Level = Level::Debug;

#[cfg(not(feature = "wasm"))]
mod timer {
    use super::*;
    use std::time::Instant;

    pub struct Timer<'a> {
        name: &'a str,
        start: Option<Instant>,
    }

    impl<'a> Timer<'a> {
        pub fn new(name: &'a str) -> Timer<'a> {
            let start = log_enabled!(TIMER_LEVEL).then(|| Instant::now());
            Timer { name, start }
        }
    }

    impl<'a> Drop for Timer<'a> {
        fn drop(&mut self) {
            if !log_enabled!(TIMER_LEVEL) {
                return;
            }
            let Some(start) = self.start else { return };

            let time = start.elapsed();
            log::debug!("Timer {} ended after {time:?}", self.name);
        }
    }
}

#[cfg(feature = "wasm")]
mod timer {
    use super::*;
    use web_sys::console;

    /// RAII wrapper around console.time()
    pub struct Timer<'a> {
        name: &'a str,
    }

    impl<'a> Timer<'a> {
        pub fn new(name: &'a str) -> Timer<'a> {
            if log_enabled!(TIMER_LEVEL) {
                console::time_with_label(name);
            }
            Timer { name }
        }
    }

    impl<'a> Drop for Timer<'a> {
        fn drop(&mut self) {
            if log_enabled!(TIMER_LEVEL) {
                console::time_end_with_label(self.name);
            }
        }
    }
}
