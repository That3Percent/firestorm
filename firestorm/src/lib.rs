//! Firestorm is the low-overhead intrusive flamegraph profiler for Rust.
//! For documentation and usage, please see [Firestorm on Github](https://github.com/That3Percent/firestorm)

#[cfg(any(feature = "enable_cpu_time", feature = "enable_system_time"))]
pub use firestorm_enabled::*;

#[cfg(not(any(feature = "enable_cpu_time", feature = "enable_system_time")))]
#[doc(hidden)]
pub mod disabled;

#[cfg(not(any(feature = "enable_cpu_time", feature = "enable_system_time")))]
pub use disabled::*;
