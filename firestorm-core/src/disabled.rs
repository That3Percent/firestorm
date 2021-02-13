// It shouldn't even be possible to bring in this module if the import root is `firestorm`,
// but the compiler doesn't know that. So, we need to create this module with a compatible API.

use std::ops::Sub;

#[derive(Copy, Clone)]
pub struct TimeSample;

impl TimeSample {
    pub fn now() -> Self {
        unreachable!()
    }
}

impl Sub for TimeSample {
    type Output = u64;
    fn sub(self, _other: TimeSample) -> Self::Output {
        unreachable!()
    }
}
