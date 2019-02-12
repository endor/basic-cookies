use std::error::Error;
use std::fmt::{Display, Error as FormatterError, Formatter};

const INTERNAL_ERROR_DESCRIPTION: &'static str = "Internal Error";

#[derive(Debug)]
pub struct InternalError(InternalErrorKind);

impl InternalError {
    pub(crate) fn new(kind: InternalErrorKind) -> InternalError {
        InternalError(kind)
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(INTERNAL_ERROR_DESCRIPTION)
    }
}

impl Error for InternalError {
    fn description(&self) -> &str {
        INTERNAL_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        None
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub(crate) enum InternalErrorKind {
    NonTerminalIndexBeyondBoundaries,
}
