// Copyright (c) 2017 CtrlC developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

mod platform {
    #[cfg(unix)]
    pub use super::unix::*;

    #[cfg(windows)]
    pub use super::windows::*;
}

use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

/// Ctrl-C error.
#[derive(Debug)]
pub enum Error {
    /// Ctrl-C signal handler already registered.
    MultipleHandlers,
    /// Unexpected system error.
    System(std::io::Error),
}

impl From<platform::Error> for Error {
    fn from(e: platform::Error) -> Error {
        let system_error = std::io::Error::new(std::io::ErrorKind::Other, e);
        Error::System(system_error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::MultipleHandlers => {
                write!(f, "Ctrl-C error: Ctrl-C signal handler already registered")
            }
            Error::System(_) => write!(f, "Ctrl-C error: Unexpected system error"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::System(ref e) => Some(e),
            _ => None,
        }
    }
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn set_handler<H>(mut handler: H) -> Result<(), Error>
where
    H: FnMut() + Send + 'static,
{
    if INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .unwrap_or_else(|e| e)
    {
        return Err(Error::MultipleHandlers);
    }

    unsafe {
        match platform::init_os_handler() {
            Ok(_) => {}
            Err(err) => {
                INITIALIZED.store(false, Ordering::SeqCst);

                return Err(err.into());
            }
        }
    }

    thread::Builder::new()
        .name("ctrl-c".into())
        .spawn(move || loop {
            unsafe {
                platform::block_ctrl_c().expect("Critical system error while waiting for Ctrl-C");
            }

            handler();
        })
        .expect("failed to spawn thread");

    Ok(())
}
