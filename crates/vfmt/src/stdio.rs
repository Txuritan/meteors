//! A simple wrapper around [`libc::write`].

/// A wrapper that calls [`libc::write`] with file descriptor 1 (aka STDOUT).
pub struct Stdout;

impl crate::uWrite for Stdout {
    type Error = ();

    fn write_str(&mut self, text: &str) -> Result<(), Self::Error> {
        let result = unsafe { libc::write(1, text.as_ptr() as *const _, text.len() as _) };

        if result < 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}

/// A wrapper that calls [`libc::write`] with file descriptor 2 (aka STDERR).
pub struct Stderr;

impl crate::uWrite for Stderr {
    type Error = ();

    fn write_str(&mut self, text: &str) -> Result<(), Self::Error> {
        let result = unsafe { libc::write(2, text.as_ptr() as *const _, text.len() as _) };

        if result < 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}

/// Prints to the standard output.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = $crate::uwrite!($crate::Stdout::new(), $($arg)*);
    };
}

/// Prints to the standard output, with a newline.
#[macro_export]
macro_rules! println {
    () => {
        let _ = $crate::uwrite!($crate::stdio::Stdout, "\n");
    };
    ($($arg:tt)*) => {
        let _ = $crate::uwriteln!($crate::stdio::Stdout, $($arg)*);
    };
}

/// Prints to the standard error.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        let _ = $crate::uwrite!($crate::Stderr::new(), $($arg)*);
    };
}

/// Prints to the standard error, with a newline.
#[macro_export]
macro_rules! eprintln {
    () => {
        let _ = $crate::uwrite!($crate::stdio::Stderr, "\n");
    };
    ($($arg:tt)*) => {
        let _ = $crate::uwriteln!($crate::stdio::Stderr, $($arg)*);
    };
}
