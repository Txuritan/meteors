// Copyright (c) 2017 CtrlC developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::{
    ffi::{c_int, c_void},
    mem,
    os::unix::io::RawFd,
};

use libc::size_t;
use nix::errno::Errno;

use super::Error as CtrlcError;

static mut PIPE: (RawFd, RawFd) = (-1, -1);

/// Platform specific error type
pub type Error = nix::Error;

/// Platform specific signal type
pub type Signal = nix::sys::signal::Signal;

// Taken from [nix](https://docs.rs/nix/0.23.0/src/nix/unistd.rs.html#992-996)
fn read(fd: RawFd, buf: &mut [u8]) -> Result<usize, Error> {
    let res = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t) };

    Errno::result(res).map(|r| r as usize)
}

// Taken from [nix](https://docs.rs/nix/0.23.0/src/nix/unistd.rs.html#1001-1005)
fn write(fd: RawFd, buf: &[u8]) -> Result<usize, Error> {
    let res = unsafe { libc::write(fd, buf.as_ptr() as *const c_void, buf.len() as size_t) };

    Errno::result(res).map(|r| r as usize)
}

// Taken from [nix](https://docs.rs/nix/0.23.0/src/nix/unistd.rs.html#1061-1071)
fn pipe() -> Result<(RawFd, RawFd), Error> {
    unsafe {
        let mut fds = mem::MaybeUninit::<[c_int; 2]>::uninit();

        let res = libc::pipe(fds.as_mut_ptr() as *mut c_int);

        Error::result(res)?;

        Ok((fds.assume_init()[0], fds.assume_init()[1]))
    }
}

// Taken from [nix](https://docs.rs/nix/0.23.0/src/nix/unistd.rs.html#984-987)
fn close(fd: RawFd) -> Result<(), Error> {
    let res = unsafe { libc::close(fd) };
    Errno::result(res).map(drop)
}

extern "C" fn os_handler(_: libc::c_int) {
    // Assuming this always succeeds. Can't really handle errors in any meaningful way.
    unsafe {
        let _ = write(PIPE.1, &[0u8]);
    }
}

// pipe2(2) is not available on macOS or iOS, so we need to use pipe(2) and fcntl(2)
#[inline]
#[cfg(any(target_os = "ios", target_os = "macos"))]
fn pipe2(flags: nix::fcntl::OFlag) -> Result<(RawFd, RawFd), Error> {
    // TODO(txuritan): remove the reest of the dependency on nix
    use nix::fcntl::{fcntl, FcntlArg, FdFlag, OFlag};

    let pipe = pipe()?;

    let mut res = Ok(0);

    if flags.contains(OFlag::O_CLOEXEC) {
        res = res
            .and_then(|_| fcntl(pipe.0, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC)))
            .and_then(|_| fcntl(pipe.1, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC)));
    }

    if flags.contains(OFlag::O_NONBLOCK) {
        res = res
            .and_then(|_| fcntl(pipe.0, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)))
            .and_then(|_| fcntl(pipe.1, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)));
    }

    match res {
        Ok(_) => Ok(pipe),
        Err(e) => {
            let _ = close(pipe.0);
            let _ = close(pipe.1);
            Err(e)
        }
    }
}

#[inline]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn pipe2(flags: nix::fcntl::OFlag) -> Result<(RawFd, RawFd), Error> {
    let mut fds = mem::MaybeUninit::<[c_int; 2]>::uninit();

    let res = unsafe { libc::pipe2(fds.as_mut_ptr() as *mut c_int, flags.bits()) };

    Errno::result(res)?;

    unsafe { Ok((fds.assume_init()[0], fds.assume_init()[1])) }
}

/// Register os signal handler.
///
/// Must be called before calling [`block_ctrl_c()`](fn.block_ctrl_c.html)
/// and should only be called once.
///
/// # Errors
/// Will return an error if a system error occurred.
///
#[inline]
pub unsafe fn init_os_handler() -> Result<(), Error> {
    use nix::fcntl;
    use nix::sys::signal;

    PIPE = pipe2(fcntl::OFlag::O_CLOEXEC)?;

    let close_pipe = |e: nix::Error| -> Error {
        // Try to close the pipes. close() should not fail,
        // but if it does, there isn't much we can do
        let _ = close(PIPE.1);
        let _ = close(PIPE.0);
        e
    };

    // Make sure we never block on write in the os handler.
    if let Err(e) = fcntl::fcntl(PIPE.1, fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_NONBLOCK)) {
        return Err(close_pipe(e));
    }

    let handler = signal::SigHandler::Handler(os_handler);
    let new_action = signal::SigAction::new(
        handler,
        signal::SaFlags::SA_RESTART,
        signal::SigSet::empty(),
    );

    let sigint_old = match signal::sigaction(signal::Signal::SIGINT, &new_action) {
        Ok(old) => old,
        Err(e) => return Err(close_pipe(e)),
    };

    // TODO: Maybe throw an error if old action is not SigDfl.

    Ok(())
}

/// Blocks until a Ctrl-C signal is received.
///
/// Must be called after calling [`init_os_handler()`](fn.init_os_handler.html).
///
/// # Errors
/// Will return an error if a system error occurred.
///
#[inline]
pub unsafe fn block_ctrl_c() -> Result<(), CtrlcError> {
    use std::io;
    let mut buf = [0u8];

    // TODO: Can we safely convert the pipe fd into a std::io::Read
    // with std::os::unix::io::FromRawFd, this would handle EINTR
    // and everything for us.
    loop {
        match read(PIPE.0, &mut buf[..]) {
            Ok(1) => break,
            Ok(_) => return Err(CtrlcError::System(io::ErrorKind::UnexpectedEof.into())),
            Err(nix::errno::Errno::EINTR) => {}
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
