//! Module for handling cookies sent from a user agent to a server.

mod cookie;
mod cookie_lexer;

pub use self::cookie::Cookie;
pub(crate) use self::cookie_lexer::{CharTokenClass, CookieLexer, CookieLexerError, CookieToken};
