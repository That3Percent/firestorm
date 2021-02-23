use std::error::Error;
use std::path::Path;

pub mod internal {
    pub struct SpanGuard;
}

#[macro_export]
macro_rules! profile_fn {
    ($($t:tt)*) => {};
}

#[macro_export]
macro_rules! profile_method {
    ($($t:tt)*) => {};
}

#[macro_export]
macro_rules! profile_section {
    ($name:ident) => {
        #[allow(unused_variables)]
        let $name = $crate::internal::SpanGuard;
    };
}

#[inline(always)]
pub fn clear() {}

#[inline(always)]
pub fn save<P: AsRef<Path>>(_path: P) -> Result<(), Box<dyn Error>> {
    Err("Firestorm not enabled")?
}

/// Returns whether or not firestorm is enabled
#[inline(always)]
pub const fn enabled() -> bool {
    false
}

pub fn bench<F: Fn(), P: AsRef<Path>>(path: P, f: F) -> Result<(), Box<dyn Error>> {
    Err("Firestorm not enabled")?
}
