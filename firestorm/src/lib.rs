#[cfg(feature = "enabled")]
extern crate inferno;

#[cfg(feature = "enabled")]
#[doc(hidden)]
pub mod enabled;

#[cfg(feature = "enabled")]
#[doc(hidden)]
pub mod internal;

#[cfg(feature = "enabled")]
pub use enabled::*;

#[cfg(not(feature = "enabled"))]
#[doc(hidden)]
pub mod disabled;

#[cfg(not(feature = "enabled"))]
pub use disabled::*;

/// Returns whether or not firestorm is enabled
#[inline(always)]
pub fn enabled() -> bool {
    cfg!(feature = "enabled")
}
