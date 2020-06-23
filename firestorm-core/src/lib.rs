use std::{cell::UnsafeCell, fmt, time::Instant};

/// A tiny record of a method call which when played back can construct
/// a profiling state.
pub enum Event {
    Start { time: Instant, tag: LazyStr },
    End { time: Instant },
    // TODO: Add Pause, Resume to help with things like the
    // amortized cost of expanding the length of the events
    // array getting reported as a part of another operation.
}

// TODO: (Performance) Optimization to avoid the cost of fat
// pointers in the profiling region.
// type Str = &'static &'static str;

/// A delayed formatting struct to move allocations out of the loop
/// This API is likely to change.
// TODO: (Performance): Try things like the Str optimization (above),
// as well as specializing a variable sized struct to write
pub enum LazyStr {
    Func(&'static str),
    Method(&'static str, &'static str),
}

impl fmt::Display for LazyStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Func(s0) => write!(f, "{}", s0),
            Self::Method(s0, s1) => write!(f, "{}::{}", s0, s1),
        }
    }
}

thread_local!(pub static EVENTS: UnsafeCell<Vec<Event>> = UnsafeCell::new(Vec::with_capacity(2048)));
