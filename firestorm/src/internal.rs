pub use firestorm_core::LazyStr;

pub struct SpanGuard;

impl Drop for SpanGuard {
    #[inline(always)]
    fn drop(&mut self) {
        crate::end();
    }
}
