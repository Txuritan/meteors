use std::{error, fmt, io, string};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8(string::FromUtf8Error),

    InvalidByte {
        expected: u8,
        got: u8,
    },
    InvalidContainer {
        expected: u8,
        got: u8,
    },
    InvalidContainers {
        expected: (u8, u8),
        got: u8,
    },

    UnknownVariant {
        expected: &'static [&'static str],
        got: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Utf8(err) => err.fmt(f),
            Error::InvalidByte { expected, got } => {
                write!(
                    f,
                    "invalid byte, expected byte `{:#x}` but got `{:#x}` instead",
                    expected, got
                )
            }
            Error::InvalidContainer { expected, got } => {
                write!(
                    f,
                    "invalid container byte, expected byte `{:#x}` but got `{:#x}` instead",
                    expected, got
                )
            }
            Error::InvalidContainers { expected, got } => {
                write!(f, "invalid container byte, expected bytes `{:#x}` or `{:#x}` but got `{:#x}` instead", expected.0, expected.1, got)
            }
            Error::UnknownVariant { expected, got } => {
                write!(f, "invalid variant, expected variants [")?;
                for (i, variant) in expected.iter().enumerate() {
                    write!(f, "`{}`", variant)?;
                    if i != expected.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "] but got `{}` instead", got)
            }
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(v: string::FromUtf8Error) -> Self {
        Self::Utf8(v)
    }
}
