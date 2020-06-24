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
        let $name = ();
    };
}

#[inline(always)]
pub fn clear() {}

#[inline(always)]
pub fn to_svg<W: std::io::Write>(_writer: &mut W) -> Result<(), std::convert::Infallible> {
    Ok(())
}
