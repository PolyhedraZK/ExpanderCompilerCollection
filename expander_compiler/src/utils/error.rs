//! This module defines the `Error` type used for error handling in the expander compiler.

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Represents an error that is caused by user input or actions.
    UserError(String),
    /// Represents an internal error that is not caused by user input, such as a bug in the code.
    /// This type of error should not occur in normal operation and indicates a problem that needs to
    /// be fixed by the developers.
    InternalError(String),
}

impl Error {
    /// Returns whether the error is a user error.
    pub fn is_user(&self) -> bool {
        matches!(self, Error::UserError(_))
    }

    /// Returns whether the error is an internal error.
    pub fn is_internal(&self) -> bool {
        matches!(self, Error::InternalError(_))
    }

    /// Prepends a prefix to the error message.
    pub fn prepend(&self, prefix: &str) -> Error {
        match self {
            Error::UserError(s) => Error::UserError(format!("{prefix}: {s}")),
            Error::InternalError(s) => Error::InternalError(format!("{prefix}: {s}")),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UserError(s) => write!(f, "{s}"),
            Error::InternalError(s) => write!(f, "{s}"),
        }
    }
}
