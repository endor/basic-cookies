#![cfg_attr(all(feature = "benchmarks", test), feature(test))]
#[cfg(all(feature = "benchmarks", test))]
pub(crate) extern crate test;

mod emit_cookie_error;
mod encoding_error;
mod indexed_string;
mod string_scanner;
mod user_agent_cookie;

pub use self::encoding_error::{EncodingError, EncodingErrorExpectedClass};
pub use self::user_agent_cookie::UserAgentCookie;

pub(crate) use self::emit_cookie_error::EmitCookieError;
pub(crate) use self::indexed_string::IndexedString;
pub(crate) use self::string_scanner::{ScanCharResult, ScanUntilCharResult, StringScanner};
