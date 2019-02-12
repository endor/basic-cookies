use std::error::Error;
use std::fmt::{Display, Error as FormatterError, Formatter};

use crate::{CookieToken, LexerError};

const PARSER_ERROR_DESCRIPTION: &'static str = "Parser Error";

pub(crate) type LalrpopError = lalrpop_util::ParseError<usize, CookieToken, LexerError>;

#[derive(Debug)]
pub struct ParserError {
    lalrpop_error: LalrpopError,
}

impl ParserError {
    pub(crate) fn new(lalrpop_error: LalrpopError) -> ParserError {
        ParserError {
            lalrpop_error: lalrpop_error,
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(PARSER_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        self.lalrpop_error.fmt(f)
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        PARSER_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.lalrpop_error)
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        Some(&self.lalrpop_error)
    }
}
