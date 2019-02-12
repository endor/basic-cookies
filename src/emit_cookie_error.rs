use std::error::Error;
use std::fmt::{Display, Error as FormatterError, Formatter};

use crate::{EncodingError, InternalError};

const EMIT_COOKIE_ERROR_DESCRIPTION: &'static str = "Error Emitting Cookie String";

#[derive(Debug)]
pub enum EmitCookieError<'a> {
    InternalError(InternalError),
    EncodingError(EncodingError<'a>),
}

impl<'a> Display for EmitCookieError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(EMIT_COOKIE_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        match self {
            EmitCookieError::InternalError(err) => err.fmt(f),
            EmitCookieError::EncodingError(err) => err.fmt(f),
        }
    }
}

impl<'a> Error for EmitCookieError<'a> {
    fn description(&self) -> &str {
        EMIT_COOKIE_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        self.source()
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        match self {
            EmitCookieError::InternalError(err) => Some(err),
            EmitCookieError::EncodingError(err) => Some(err.get_owned()),
        }
    }
}
