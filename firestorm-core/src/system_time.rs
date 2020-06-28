use std::{ops::Sub, time::Instant};

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
