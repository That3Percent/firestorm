#[cfg(any(feature = "enable_cpu_time", feature = "enable_system_time"))]
extern crate inferno;

#[cfg(any(feature = "enable_cpu_time", feature = "enable_system_time"))]
#[doc(hidden)]
pub mod enabled;

#[cfg(any(feature = "enable_cpu_time", feature = "enable_system_time"))]
#[doc(hidden)]
pub mod internal;

#[cfg(any(feature = "enable_cpu_time", feature = "enable_system_time"))]
pub use enabled::*;

#[cfg(not(any(feature = "enable_cpu_time", feature = "enable_system_time")))]
#[doc(hidden)]
pub mod disabled;

#[cfg(not(any(feature = "enable_cpu_time", feature = "enable_system_time")))]
pub use disabled::*;

/// Returns whether or not firestorm is enabled
#[inline(always)]
pub fn enabled() -> bool {
    cfg!(any(
        feature = "enable_cpu_time",
        feature = "enable_system_time"
    ))
}
