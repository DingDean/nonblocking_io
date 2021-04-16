use std::time::{Duration, SystemTime};

pub struct Timer {
    pub deadline: SystemTime,
    pub cb: Box<dyn FnOnce()>,
}

impl Timer {
    pub fn new(t: Duration, cb: Box<dyn FnOnce()>) -> Self {
        Self {
            deadline: SystemTime::now() + t,
            cb,
        }
    }
}

fn run(t: Timer) {
    (t.cb)();
}

fn main() {
    let timer = Timer::new(
        Duration::from_secs(1),
        Box::new(|| {
            println!("hello world");
        }),
    );
    run(timer);
}
