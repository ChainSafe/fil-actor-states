// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::error::ExitCode;
use thiserror::Error;

/// The error type returned by actor method calls.
#[derive(Error, Debug, Clone, PartialEq)]
#[error("ActorError(exit_code: {exit_code:?}, msg: {msg})")]
pub struct ActorError {
    /// The exit code for this invocation.
    /// Codes less than `FIRST_ACTOR_EXIT_CODE` are prohibited and will be overwritten by the VM.
    exit_code: ExitCode,
    /// Message for debugging purposes,
    msg: String,
}

/// Convenience macro for generating Actor Errors
#[macro_export]
macro_rules! actor_error_v8 {
    // Error with only one stringable expression
    ( $code:ident; $msg:expr ) => { $crate::v8::ActorError::$code($msg.to_string()) };

    // String with positional arguments
    ( $code:ident; $msg:literal $(, $ex:expr)+ ) => {
        $crate::v8::ActorError::$code(format!($msg, $($ex,)*))
    };

    // Error with only one stringable expression, with comma separator
    ( $code:ident, $msg:expr ) => { $crate::actor_error_v8!($code; $msg) };

    // String with positional arguments, with comma separator
    ( $code:ident, $msg:literal $(, $ex:expr)+ ) => {
        $crate::actor_error_v8!($code; $msg $(, $ex)*)
    };
}

impl ActorError {
    /// Creates a new `ActorError`. This method does not check that the code is in the
    /// range of valid actor abort codes.
    pub fn unchecked(code: ExitCode, msg: String) -> Self {
        Self {
            exit_code: code,
            msg,
        }
    }

    pub fn illegal_argument(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_ILLEGAL_ARGUMENT,
            msg,
        }
    }
    pub fn not_found(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_NOT_FOUND,
            msg,
        }
    }
    pub fn forbidden(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_FORBIDDEN,
            msg,
        }
    }
    pub fn insufficient_funds(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_INSUFFICIENT_FUNDS,
            msg,
        }
    }
    pub fn illegal_state(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_ILLEGAL_STATE,
            msg,
        }
    }
    pub fn serialization(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_SERIALIZATION,
            msg,
        }
    }
    pub fn unhandled_message(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_UNHANDLED_MESSAGE,
            msg,
        }
    }
    pub fn unspecified(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_UNSPECIFIED,
            msg,
        }
    }
    pub fn assertion_failed(msg: String) -> Self {
        Self {
            exit_code: ExitCode::USR_ASSERTION_FAILED,
            msg,
        }
    }

    /// Returns the exit code of the error.
    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }

    /// Error message of the actor error.
    pub fn msg(&self) -> &str {
        &self.msg
    }

    /// Prefix error message with a string message.
    pub fn wrap(mut self, msg: impl AsRef<str>) -> Self {
        self.msg = format!("{}: {}", msg.as_ref(), self.msg);
        self
    }
}

/// Converts a raw encoding error into an `ErrSerialization`.
impl From<fvm_ipld_encoding::Error> for ActorError {
    fn from(e: fvm_ipld_encoding::Error) -> Self {
        Self {
            exit_code: ExitCode::USR_SERIALIZATION,
            msg: e.to_string(),
        }
    }
}
