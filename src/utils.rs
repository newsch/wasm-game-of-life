pub use timer::Timer;

#[cfg(not(feature = "wasm"))]
mod timer {
    use std::time::Instant;

    pub struct Timer<'a> {
        name: &'a str,
        start: Instant,
    }

    impl<'a> Timer<'a> {
        pub fn new(name: &'a str) -> Timer<'a> {
            let start = Instant::now();
            Timer { name, start }
        }
    }

    impl<'a> Drop for Timer<'a> {
        fn drop(&mut self) {
            let time = self.start.elapsed();
            eprintln!("Timer {} ended after {time:?}", self.name);
        }
    }
}

#[cfg(feature = "wasm")]
mod timer {
    use web_sys::console;

    /// RAII wrapper around console.time()
    pub struct Timer<'a> {
        name: &'a str,
    }

    impl<'a> Timer<'a> {
        pub fn new(name: &'a str) -> Timer<'a> {
            console::time_with_label(name);
            Timer { name }
        }
    }

    impl<'a> Drop for Timer<'a> {
        fn drop(&mut self) {
            console::time_end_with_label(self.name);
        }
    }
}
