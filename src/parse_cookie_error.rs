use std::error::Error;
use std::fmt::{Display, Error as FormatterError, Formatter};

use crate::{InternalError, LalrpopError, ParserError};

const PARSE_COOKIE_ERROR_DESCRIPTION: &'static str = "Error Parsing Cookie String";

#[derive(Debug)]
pub enum ParseCookieError {
    InternalError(InternalError),
    ParserError(ParserError),
}

impl ParseCookieError {
    pub(crate) fn from_lalrpop_error(err: LalrpopError) -> ParseCookieError {
        ParseCookieError::ParserError(ParserError::new(err))
    }

    pub(crate) fn from_internal_error(err: InternalError) -> ParseCookieError {
        ParseCookieError::InternalError(err)
    }
}

impl Display for ParseCookieError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(PARSE_COOKIE_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        match self {
            ParseCookieError::InternalError(err) => err.fmt(f),
            ParseCookieError::ParserError(err) => err.fmt(f),
        }
    }
}

impl Error for ParseCookieError {
    fn description(&self) -> &str {
        PARSE_COOKIE_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        self.source()
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        match self {
            ParseCookieError::InternalError(err) => Some(err),
            ParseCookieError::ParserError(err) => Some(err),
        }
    }
}
