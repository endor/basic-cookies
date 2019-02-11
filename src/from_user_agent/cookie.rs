use super::{CharTokenClass, CookieLexer, CookieLexerError, CookieToken};
use crate::lalrpop_util;
use std::fmt::{Display, Error as FormatterError, Formatter};

const BASIC_COOKIE_ERROR_DESCRIPTION: &'static str = "Cookie Parsing Error";
const INTERNAL_ERROR_DESCRIPTION: &'static str = "Internal Error";
const PARSE_ERROR_DESCRIPTION: &'static str = "Parse Error";
const ENCODING_ERROR_DESCRIPTION: &'static str = "Encoding Error";

const ENCODING_ERROR_TOKEN_CLASS: &'static str = "token";
const ENCODING_ERROR_COOKIE_OCTET_CLASS: &'static str = "cookie-octet";

#[allow(dead_code)]
lalrpop_mod!(cookie_grammar, "/from_user_agent/cookie_grammar.rs");

/// A cookie sent from a user agent to a server, as described in [Section 4.2 of RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2).
///
/// # Examples
/// ```
/// use basic_cookies::from_user_agent::Cookie;
///
/// let new_cookie_0 = Cookie::new("key0", "value0");
/// let new_cookie_1 = Cookie::new("key1", "value1");
/// let cookie_string = Cookie::emit(vec![new_cookie_0, new_cookie_1]).unwrap();
/// assert_eq!("key0=value0; key1=value1", cookie_string);
///
/// let parsed_cookies = Cookie::parse(&cookie_string).unwrap();
/// assert_eq!("key0", parsed_cookies[0].get_name());
/// assert_eq!("value0", parsed_cookies[0].get_value());
/// assert_eq!("key1", parsed_cookies[1].get_name());
/// assert_eq!("value1", parsed_cookies[1].get_value());
/// ```
#[derive(Debug)]
pub struct Cookie<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> Cookie<'a> {
    /// Creates a new cookie from a cookie name string and a cookie value string.
    ///
    /// # Examples
    /// ```
    /// use basic_cookies::from_user_agent::Cookie;
    ///
    /// let new_cookie = Cookie::new("name1", "value1");
    /// assert_eq!("name1", new_cookie.get_name());
    /// assert_eq!("value1", new_cookie.get_value());
    /// ```
    pub fn new(name: &'a str, value: &'a str) -> Cookie<'a> {
        Cookie {
            name: name,
            value: value,
        }
    }

    /// Parses an [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) compliant cookie string, sent from a user agent.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::from_user_agent::Cookie;
    ///
    /// let parsed_cookies = Cookie::parse("cookie1=value1; cookie2=value2").unwrap();
    ///
    /// assert_eq!("cookie1", parsed_cookies[0].get_name());
    /// assert_eq!("value1", parsed_cookies[0].get_value());
    ///
    /// assert_eq!("cookie2", parsed_cookies[1].get_name());
    /// assert_eq!("value2", parsed_cookies[1].get_value());
    /// ```
    pub fn parse(input: &'a str) -> Result<Vec<Cookie<'a>>, ParseError> {
        Ok(cookie_grammar::CookiesParser::new()
            .parse(CookieLexer::new(input))
            .map_err(ParserError::from_lalrpop_parse_error_to_error)?
            .iter()
            .map(|tok| tok.with_str(input))
            .collect::<Result<Vec<Cookie>, ParseError>>()?)
    }

    /// Gets the name of the cookie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::from_user_agent::Cookie;
    ///
    /// let parsed_cookies = Cookie::parse("name=value").unwrap();
    /// assert_eq!("name", parsed_cookies[0].get_name());
    /// ```
    pub fn get_name(&self) -> &'a str {
        self.name
    }

    /// Gets the value of the cookie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::from_user_agent::Cookie;
    ///
    /// let parsed_cookies = Cookie::parse("name=value").unwrap();
    /// assert_eq!("value", parsed_cookies[0].get_value());
    /// ```
    pub fn get_value(&self) -> &'a str {
        self.value
    }

    /// Emits [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) a compliant cookie string, suitable to be sent from a user agent to a server.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::from_user_agent::Cookie;
    ///
    /// let cookie_0 = Cookie::new("key0", "value0");
    /// let cookie_1 = Cookie::new("key1", "value1");
    /// let cookie_string = Cookie::emit(vec![cookie_0, cookie_1]).unwrap();
    /// assert_eq!("key0=value0; key1=value1", cookie_string);
    /// ```
    pub fn emit<T: IntoIterator<Item = Cookie<'a>>>(cookies: T) -> Result<String, EmitError> {
        let mut result = String::new();
        let mut is_first = true;

        for cookie in cookies {
            if is_first {
                is_first = false;
            } else {
                result.push_str("; ");
            }

            if !is_str_all_tokens(cookie.name) {
                return Err(EmitError::EncodingError(EncodingError::new(
                    cookie.name.to_owned(),
                    ENCODING_ERROR_TOKEN_CLASS,
                )));
            }

            if !is_str_all_cookie_octets(cookie.value) {
                return Err(EmitError::EncodingError(EncodingError::new(
                    cookie.value.to_owned(),
                    ENCODING_ERROR_COOKIE_OCTET_CLASS,
                )));
            }

            result.push_str(cookie.name);
            result.push('=');
            result.push_str(cookie.value);
        }

        Ok(result)
    }
}

#[derive(Debug)]
pub enum ParseError {
    InternalError(InternalError),
    ParserError(ParserError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(BASIC_COOKIE_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        match self {
            ParseError::InternalError(err) => err.fmt(f),
            ParseError::ParserError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        BASIC_COOKIE_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&std::error::Error> {
        self.source()
    }

    fn source(&self) -> Option<&(std::error::Error + 'static)> {
        match self {
            ParseError::InternalError(err) => Some(err),
            ParseError::ParserError(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum EmitError {
    InternalError(InternalError),
    EncodingError(EncodingError),
}

#[derive(Debug)]
pub struct InternalError(InternalErrorKind);

impl InternalError {
    pub(crate) fn to_error(self) -> ParseError {
        ParseError::InternalError(self)
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(INTERNAL_ERROR_DESCRIPTION)
    }
}

impl std::error::Error for InternalError {
    fn description(&self) -> &str {
        INTERNAL_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
enum InternalErrorKind {
    NonTerminalIndexBeyondBoundaries,
}

type LalrpopError = lalrpop_util::ParseError<usize, CookieToken, CookieLexerError>;

#[derive(Debug)]
pub struct ParserError {
    lalrpop_error: LalrpopError,
}

impl ParserError {
    pub(crate) fn from_lalrpop_parse_error_to_error(src: LalrpopError) -> ParseError {
        ParserError { lalrpop_error: src }.to_error()
    }

    fn to_error(self) -> ParseError {
        ParseError::ParserError(self)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_str(PARSE_ERROR_DESCRIPTION)?;
        f.write_str(": ")?;
        self.lalrpop_error.fmt(f)
    }
}

impl std::error::Error for ParserError {
    fn description(&self) -> &str {
        PARSE_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&std::error::Error> {
        Some(&self.lalrpop_error)
    }

    fn source(&self) -> Option<&(std::error::Error + 'static)> {
        Some(&self.lalrpop_error)
    }
}

#[derive(Debug)]
pub struct EncodingError {
    value: String,
    expected_class: &'static str,
}

impl EncodingError {
    fn new(value: String, expected_class: &'static str) -> EncodingError {
        EncodingError {
            value: value,
            expected_class: expected_class,
        }
    }
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatterError> {
        f.write_fmt(format_args!(
            "{}, expected character class: {}, value: {}",
            ENCODING_ERROR_DESCRIPTION, &self.expected_class, &self.value
        ))
    }
}

impl std::error::Error for EncodingError {
    fn description(&self) -> &str {
        ENCODING_ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(std::error::Error + 'static)> {
        None
    }
}

fn is_str_all_tokens(val: &str) -> bool {
    val.chars().all(CharTokenClass::is_token_char)
}

fn is_str_all_cookie_octets(val: &str) -> bool {
    val.chars().all(CharTokenClass::is_cookie_octet)
}

mod terminals {
    use super::nonterminals::NonTerminalSpan;
    use super::Cookie as FullyParsedCookie;
    use super::{InternalError, ParseError};

    #[derive(Clone, Debug)]
    pub struct Cookie {
        pub(super) key: NonTerminalSpan,
        pub(super) value: NonTerminalSpan,
    }

    impl Cookie {
        pub(super) fn with_str<'a>(
            &self,
            data: &'a str,
        ) -> Result<FullyParsedCookie<'a>, ParseError> {
            Ok(FullyParsedCookie::new(
                self.key.as_str(data).map_err(InternalError::to_error)?,
                self.value.as_str(data).map_err(InternalError::to_error)?,
            ))
        }
    }
}

mod nonterminals {
    use super::{InternalError, InternalErrorKind};

    #[derive(Clone, Debug)]
    pub struct NonTerminalSpan {
        start: usize,
        end: usize,
    }

    impl NonTerminalSpan {
        pub(crate) fn new(start: usize, end: usize) -> NonTerminalSpan {
            NonTerminalSpan {
                start: start,
                end: end,
            }
        }

        pub(crate) fn as_str<'a>(&self, data: &'a str) -> Result<&'a str, InternalError> {
            match data.get(self.start..self.end) {
                Some(res) => Ok(res),
                None => Err(InternalError(
                    InternalErrorKind::NonTerminalIndexBeyondBoundaries,
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_str_all_cookie_octets, is_str_all_tokens, Cookie};

    #[test]
    fn get_name() {
        const COOKIE_KEY: &'static str = "cookie_key";
        const COOKIE_VALUE: &'static str = "cookie_value";

        let cookie = Cookie {
            name: COOKIE_KEY,
            value: COOKIE_VALUE,
        };

        assert_eq!(COOKIE_KEY, cookie.get_name());
    }

    #[test]
    fn get_value() {
        const COOKIE_KEY: &'static str = "cookie_key";
        const COOKIE_VALUE: &'static str = "cookie_value";

        let cookie = Cookie {
            name: COOKIE_KEY,
            value: COOKIE_VALUE,
        };

        assert_eq!(COOKIE_VALUE, cookie.get_value());
    }

    #[test]
    fn single_cookie() {
        const COOKIE_STR: &'static str = "test=1234";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("1234", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_quoted() {
        const COOKIE_STR: &'static str = "quoted_test=\"quotedval\"";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("quoted_test", parsed_cookie.name);
        assert_eq!("quotedval", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_with_equals_in_value() {
        const COOKIE_STR: &'static str = "test=abc=123";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("abc=123", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_before() {
        const COOKIE_STR: &'static str = " qtest=9878";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("qtest", parsed_cookie.name);
        assert_eq!("9878", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_after() {
        const COOKIE_STR: &'static str = "abcde=77766test \x09\x09    ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("abcde", parsed_cookie.name);
        assert_eq!("77766test", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_after() {
        const COOKIE_STR: &'static str = "xyzzz=test3 ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("xyzzz", parsed_cookie.name);
        assert_eq!("test3", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before_and_after() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876       ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name() {
        const COOKIE_STR: &'static str = "=nokey";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_with_ows_before() {
        const COOKIE_STR: &'static str = " =nokey";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value() {
        const COOKIE_STR: &'static str = "noval=";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value_with_ows_after() {
        const COOKIE_STR: &'static str = "noval= ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_and_val() {
        const COOKIE_STR: &'static str = "=";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_no_equals() {
        const COOKIE_STR: &'static str = "nokey";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn two_cookies() {
        const COOKIE_STR: &'static str = "test1=01234; test2=testval";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(2, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("01234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("testval", parsed_cookie_1.value);
    }

    #[test]
    fn three_cookies() {
        const COOKIE_STR: &'static str = "test1=0x1234; test2=test2; third_val=v4lue";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_ows_before() {
        const COOKIE_STR: &'static str = " test1=0x1234; test2=test2; third_val=v4lue";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_ows_after() {
        const COOKIE_STR: &'static str = "test1=0x1234; test2=test2; third_val=v4lue   ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn three_cookies_ows_before_and_after() {
        const COOKIE_STR: &'static str = "   test1=0x1234; test2=test2; third_val=v4lue ";
        let parsed_cookies = Cookie::parse(COOKIE_STR).unwrap();
        assert_eq!(3, parsed_cookies.len());

        let parsed_cookie_0 = &parsed_cookies[0];
        assert_eq!("test1", parsed_cookie_0.name);
        assert_eq!("0x1234", parsed_cookie_0.value);

        let parsed_cookie_1 = &parsed_cookies[1];
        assert_eq!("test2", parsed_cookie_1.name);
        assert_eq!("test2", parsed_cookie_1.value);

        let parsed_cookie_2 = &parsed_cookies[2];
        assert_eq!("third_val", parsed_cookie_2.name);
        assert_eq!("v4lue", parsed_cookie_2.value);
    }

    #[test]
    fn is_str_all_tokens_empty() {
        assert_eq!(true, is_str_all_tokens(""));
    }

    #[test]
    fn is_str_all_tokens_true() {
        assert_eq!(true, is_str_all_tokens("hello"));
    }

    #[test]
    fn is_str_all_tokens_false() {
        assert_eq!(false, is_str_all_tokens("[hello]"));
    }

    #[test]
    fn is_str_all_cookie_octets_empty() {
        assert_eq!(true, is_str_all_cookie_octets(""));
    }

    #[test]
    fn is_str_all_cookie_octets_true() {
        assert_eq!(true, is_str_all_cookie_octets("hello"));
    }

    #[test]
    fn is_str_all_cookie_octets_true_with_non_token_chars() {
        assert_eq!(true, is_str_all_cookie_octets("[hello]"));
    }

    #[test]
    fn is_str_all_cookie_octets_false() {
        assert_eq!(false, is_str_all_cookie_octets("=hello"));
    }

    #[test]
    fn emit_empty() {
        assert_eq!("", Cookie::emit(vec![]).unwrap());
    }

    #[test]
    fn emit_single() {
        assert_eq!(
            "testkey=testvalue",
            Cookie::emit(vec![Cookie::new("testkey", "testvalue")]).unwrap()
        );
    }

    #[test]
    fn emit_two() {
        assert_eq!(
            "abc=123; hello=world",
            Cookie::emit(vec![
                Cookie::new("abc", "123"),
                Cookie::new("hello", "world")
            ])
            .unwrap()
        );
    }

    #[test]
    fn emit_invalid_token() {
        assert!(
            Cookie::emit(vec![Cookie::new("[abc]", "123")]).is_err(),
            "EncodingError expected but result was successful."
        );
    }

    #[test]
    fn emit_invalid_cookie_value() {
        assert!(
            Cookie::emit(vec![Cookie {
                name: "abc",
                value: "\"123\""
            }])
            .is_err(),
            "EncodingError expected but result was successful."
        );
    }
}
