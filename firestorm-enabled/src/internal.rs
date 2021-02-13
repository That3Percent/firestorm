use crate::*;
pub use firestorm_core::{EventData, Start};

pub struct SpanGuard;

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
