use std::error::Error;
use std::fmt::{Display, Error as FormatterError, Formatter};
const LEXER_ERROR_DESCRIPTION: &'static str = "Lexer Error";

#[derive(Debug)]
pub struct LexerError;

impl PartialEq<LexerError> for LexerError {
    fn eq(&self, _other: &LexerError) -> bool {
        false
    }

    fn ne(&self, _other: &LexerError) -> bool {
        false
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(LEXER_ERROR_DESCRIPTION)
    }
}

impl Error for LexerError {
    fn description(&self) -> &str {
        LEXER_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        None
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        None
    }
}
