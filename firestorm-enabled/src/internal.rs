use crate::*;
pub use firestorm_core::{EventData, Start};
use std::marker::PhantomData;

// See also 6cb371a0-d6f0-48be-87d2-8d824b82e0e7
// This is a hack to impl !Send for SpanGuard.
// The reason we want to do this is to prevent
// the SpanGuard to be sent to another thread
// where it can pop a scope that wasn't pushed
// into the same collection (which would eventually
// result in a panic or some very misleading data)
// Generally the library doesn't force you to not do this -
// you can still go out of your way to create a problem. But,
// this prevents an easy mistake profiling an `async fn`. It is
// still possible to profile an async fn, but the future won't impl
// Send, which prevents this problem.
struct Unsend(PhantomData<*const ()>);
impl Unsend {
    #[inline(always)]
    fn new() -> Self {
        Self(PhantomData)
    }
}
unsafe impl Sync for Unsend {}
pub struct SpanGuard(Unsend);
impl SpanGuard {
    #[inline(always)]
    pub fn new() -> Self {
        Self(Unsend::new())
    }
}

impl Drop for SpanGuard {
    #[inline(always)]
    fn drop(&mut self) {
        crate::end();
    }
}

pub fn start(data: EventData) {
    with_events(|events| {
        let event = Event {
            time: TimeSample::now(),
            data,
        };
        events.push(event);
    });
}
