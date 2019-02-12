use std::error::Error;
use std::fmt::{Display, Error as FormatterError, Formatter};

const ENCODING_ERROR_DESCRIPTION: &'static str = "Encoding Error";

#[derive(Clone, Copy, Debug)]
pub enum EncodingErrorExpectedClass {
    Token,
    CookieOctet,
}

#[derive(Debug)]
pub struct EncodingError<'a> {
    value: &'a str,
    owned_data: OwnedEncodingError,
}

impl<'a> EncodingError<'a> {
    pub(crate) fn new(
        value: &'a str,
        expected_class: EncodingErrorExpectedClass,
    ) -> EncodingError<'a> {
        EncodingError {
            value: value,
            owned_data: OwnedEncodingError {
                expected_class: expected_class,
            },
        }
    }

    pub(crate) fn get_owned<'s>(&'s self) -> &'s OwnedEncodingError {
        &self.owned_data
    }
}

impl<'a> Display for EncodingError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_fmt(format_args!(
            "{}, expected character class: {:?}, value: {}",
            ENCODING_ERROR_DESCRIPTION, &self.owned_data.expected_class, self.value
        ))
    }
}

impl<'a> Error for EncodingError<'a> {
    fn description(&self) -> &str {
        ENCODING_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        None
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub(crate) struct OwnedEncodingError {
    expected_class: EncodingErrorExpectedClass,
}

impl Display for OwnedEncodingError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_fmt(format_args!(
            "{}, expected character class: {:?}",
            ENCODING_ERROR_DESCRIPTION, &self.expected_class
        ))
    }
}

impl Error for OwnedEncodingError {
    fn description(&self) -> &str {
        ENCODING_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&Error> {
        None
    }

    fn source(&self) -> Option<&(Error + 'static)> {
        None
    }
}
