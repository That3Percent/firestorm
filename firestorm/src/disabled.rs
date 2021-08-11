use std::error::Error;
use std::path::Path;

pub mod internal {
    use std::marker::PhantomData;
    // See also 6cb371a0-d6f0-48be-87d2-8d824b82e0e7
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

    // Ensure that SpanGuard implements the same traits as the enabled version
    impl Drop for SpanGuard {
        #[inline(always)]
        fn drop(&mut self) {}
    }
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
        let $name = $crate::internal::SpanGuard::new();
    };
}

#[inline(always)]
pub fn clear() {}

pub fn save<P: AsRef<Path>>(_path: P) -> Result<(), Box<dyn Error>> {
    Err("Firestorm not enabled")?
}

/// Returns whether or not firestorm is enabled
#[inline(always)]
pub const fn enabled() -> bool {
    false
}

pub fn bench<F: Fn(), P: AsRef<Path>>(_path: P, _f: F) -> Result<(), Box<dyn Error>> {
    Err("Firestorm not enabled")?
}
