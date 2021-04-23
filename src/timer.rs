use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Timer {
    pub deadline: SystemTime,
    pub cb: Box<dyn Fn()>,
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deadline.cmp(&other.deadline)
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.deadline.cmp(&other.deadline))
    }
}
impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl Eq for Timer {}

impl Timer {
    pub fn new(t: Duration, cb: Box<dyn Fn()>) -> Self {
        Self {
            deadline: SystemTime::now() + t,
            cb,
        }
    }
}

pub struct Queue {
    due: Option<SystemTime>,
    inner: BinaryHeap<Reverse<Timer>>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            due: None,
            inner: BinaryHeap::new(),
        }
    }

    pub fn add(&mut self, t: Duration, cb: Box<dyn Fn()>) {
        self.inner.push(Reverse(Timer::new(t, cb)));
        let due = match self.inner.peek() {
            Some(Reverse(a)) => Some(a.deadline),
            None => None,
        };
        self.due = due;
        println!("{:?}", due);
    }

    pub fn run(&mut self) -> Option<Duration> {
        let now = SystemTime::now();

        if let Some(t) = self.due {
            if t.le(&now) {
                // 执行所有需要执行的回调
                loop {
                    let timer = self.inner.pop();
                    match timer {
                        Some(Reverse(timer)) => {
                            if timer.deadline.le(&now) {
                                (timer.cb)();
                                break;
                            }
                        }
                        None => break,
                    }
                }
                // 更新 due
                match self.inner.peek() {
                    Some(Reverse(t)) => {
                        self.due = Some(t.deadline);
                        return t
                            .deadline
                            .duration_since(now)
                            .map_or_else(|_| None, |a| Some(a));
                    }
                    None => {
                        self.due = None;
                        return None;
                    }
                }
            } else {
                return t.duration_since(now).map_or_else(|_| None, |a| Some(a));
            }
        }
        None
    }
}
