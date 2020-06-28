use std::cell::UnsafeCell;

// The TimeSample type exists to make time opaque for backward compatability.
// The means by which time is sampled has a huge impact on the performance of this
// crate. So, it is desirable to be able to change the method without requiring
// a version bump.
//
// We also make the method configurable

#[cfg(feature = "cpu_time")]
mod cpu_time;
#[cfg(feature = "cpu_time")]
pub use cpu_time::*;

#[cfg(feature = "system_time")]
mod system_time;
#[cfg(feature = "system_time")]
pub use system_time::*;

type Str = &'static &'static str;

// TODO: Add Pause, Resume to help with things like the
// amortized cost of expanding the length of the events
// array getting reported as a part of another operation.
#[non_exhaustive]
pub enum Start {
    Method { typ: &'static str, signature: Str },
    Func { signature: Str },
    Section { name: Str },
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
/// Ideally, data would be an &'static EventData to avoid writing more data
/// than is necessary but limitations in the Rust compiler prevent this.
pub struct Event {
    #[cfg(any(feature = "system_time", feature = "cpu_time"))]
    pub time: TimeSample,
    pub data: EventData,
}

// Having a large capacity here buys some time before having to implement Pause/Resume capabilities to hide
// the time spent in expanding the array.
thread_local!(pub static EVENTS: UnsafeCell<Vec<Event>> = UnsafeCell::new(Vec::with_capacity(8192)));
