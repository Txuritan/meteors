macro_rules! assume_unreachable {
    () => {
        if cfg!(debug_assertions) {
            unreachable!()
        } else {
            core::hint::unreachable_unchecked()
        }
    };
}

/// A replacement for [`std::format`]
#[cfg(feature = "std")]
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        let mut buf = String::new();
        let _ = $crate::uwrite!(&mut buf, $($arg)*);
        buf
    }};
}
