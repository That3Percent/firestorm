use std::ops::Sub;
use std::{cell::UnsafeCell, time::Instant};

/// This type exists to make time opaque for backward compatability.
/// The means by which time is sampled has a huge impact on the performance of this
/// crate. So, it is desirable to be able to change the method without requiring
/// a version bump.
// TODO: (Performance) Consider https://crates.io/crates/amd64_timer
#[derive(Copy, Clone)]
pub struct TimeSample(Instant);

impl TimeSample {
    #[inline(always)]
    pub fn now() -> Self {
        Self(Instant::now())
    }
}

impl Sub for TimeSample {
    type Output = u64;
    fn sub(self, rhs: TimeSample) -> Self::Output {
        // Assuming that this runs for less than 500 years.
        (self.0 - rhs.0).as_nanos() as u64
    }
}

// TODO: Add Pause, Resume to help with things like the
// amortized cost of expanding the length of the events
// array getting reported as a part of another operation.

#[non_exhaustive]
pub enum Start {
    Method {
        typ: &'static str,
        signature: &'static str,
    },
    Func {
        signature: &'static str,
    },
    Section {
        name: &'static str,
    },
}

/// A lazy string format.
#[non_exhaustive]
pub enum EventData {
    Start(Start),
    End,
}

/// A tiny record of a method call which when played back can construct
/// a profiling state. Several representations were considered, the most
/// elaborate of which would write variable length data to the event stream.
/// This representation allows for very simple extension of EventData without
/// increasing the cost of writing events in the common case.
pub struct Event {
    pub time: TimeSample,
    pub data: &'static EventData,
}

// Having a large capacity here buys some time before having to implement Pause/Resume capabilities to hide
// the time spent in expanding the array.
thread_local!(pub static EVENTS: UnsafeCell<Vec<Event>> = UnsafeCell::new(Vec::with_capacity(8192)));
