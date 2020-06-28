use amd64_timer::ticks;
use std::ops::Sub;

#[derive(Copy, Clone)]
pub struct TimeSample(u64);

impl TimeSample {
    #[inline(always)]
    pub fn now() -> Self {
        Self(ticks())
    }
}

impl Sub for TimeSample {
    type Output = u64;
    fn sub(self, rhs: TimeSample) -> Self::Output {
        self.0 - rhs.0
    }
}
