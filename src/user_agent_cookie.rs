use crate::{
    CharTokenClass, CookieLexer, EmitCookieError, EncodingError, EncodingErrorExpectedClass,
    ParseCookieError,
};

#[allow(dead_code)]
lalrpop_mod!(cookie_grammar, "/user_agent_cookie_grammar.rs");

/// A cookie suitable to be sent from a user agent to a server, as described in [Section 4.2 of RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2).
///
/// # Examples
/// ```
/// use basic_cookies::UserAgentCookie;
///
/// let new_cookie_0 = UserAgentCookie::new("key0", "value0");
/// let new_cookie_1 = UserAgentCookie::new("key1", "value1");
/// let cookie_string = UserAgentCookie::emit_all(vec![new_cookie_0, new_cookie_1]).unwrap();
/// assert_eq!("key0=value0; key1=value1", &cookie_string);
///
/// let parsed_cookies = UserAgentCookie::parse(&cookie_string).unwrap();
/// assert_eq!("key0", parsed_cookies[0].get_name());
/// assert_eq!("value0", parsed_cookies[0].get_value());
/// assert_eq!("key1", parsed_cookies[1].get_name());
/// assert_eq!("value1", parsed_cookies[1].get_value());
/// ```
#[derive(Debug)]
pub struct UserAgentCookie<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> UserAgentCookie<'a> {
    /// Creates a new cookie suitable to be sent from a user agent to a server from a cookie name string and a cookie value string.
    ///
    /// # Examples
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let new_cookie = UserAgentCookie::new("name1", "value1");
    /// assert_eq!("name1", new_cookie.get_name());
    /// assert_eq!("value1", new_cookie.get_value());
    /// ```
    pub fn new(name: &'a str, value: &'a str) -> UserAgentCookie<'a> {
        UserAgentCookie {
            name: name,
            value: value,
        }
    }

    /// Parses an [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) compliant cookie string, sent from a user agent.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let parsed_cookies = UserAgentCookie::parse("cookie1=value1; cookie2=value2").unwrap();
    ///
    /// assert_eq!("cookie1", parsed_cookies[0].get_name());
    /// assert_eq!("value1", parsed_cookies[0].get_value());
    ///
    /// assert_eq!("cookie2", parsed_cookies[1].get_name());
    /// assert_eq!("value2", parsed_cookies[1].get_value());
    /// ```
    pub fn parse(input: &'a str) -> Result<Vec<UserAgentCookie<'a>>, ParseCookieError> {
        Ok(cookie_grammar::CookiesParser::new()
            .parse(CookieLexer::new(input))
            .map_err(ParseCookieError::from_lalrpop_error)?
            .iter()
            .map(|tok| tok.with_str(input))
            .collect::<Result<Vec<UserAgentCookie>, ParseCookieError>>()?)
    }

    /// Gets the name of the cookie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let parsed_cookies = UserAgentCookie::parse("name=value").unwrap();
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
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let parsed_cookies = UserAgentCookie::parse("name=value").unwrap();
    /// assert_eq!("value", parsed_cookies[0].get_value());
    /// ```
    pub fn get_value(&self) -> &'a str {
        self.value
    }

    /// Emits an [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) a compliant cookie string comprised of
    /// multiple cookies, suitable to be sent from a user agent to a server.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let cookie_0 = UserAgentCookie::new("key0", "value0");
    /// let cookie_1 = UserAgentCookie::new("key1", "value1");
    /// let cookie_string = UserAgentCookie::emit_all(vec![cookie_0, cookie_1]).unwrap();
    /// assert_eq!("key0=value0; key1=value1", cookie_string);
    /// ```
    pub fn emit_all<T: IntoIterator<Item = UserAgentCookie<'a>>>(
        cookies: T,
    ) -> Result<String, EmitCookieError<'a>> {
        let mut result = String::new();
        let mut is_first = true;

        for cookie in cookies {
            if is_first {
                is_first = false;
            } else {
                result.push_str("; ");
            }

            if !is_str_all_tokens(cookie.name) {
                return Err(EmitCookieError::EncodingError(EncodingError::new(
                    cookie.name,
                    EncodingErrorExpectedClass::Token,
                )));
            }

            if !is_str_all_cookie_octets(cookie.value) {
                return Err(EmitCookieError::EncodingError(EncodingError::new(
                    cookie.value,
                    EncodingErrorExpectedClass::Token,
                )));
            }

            result.push_str(cookie.name);
            result.push('=');
            result.push_str(cookie.value);
        }

        Ok(result)
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
    use super::ParseCookieError;
    use super::UserAgentCookie;

    #[derive(Clone, Debug)]
    pub struct Cookie {
        pub(super) key: NonTerminalSpan,
        pub(super) value: NonTerminalSpan,
    }

    impl Cookie {
        pub(super) fn with_str<'a>(
            &self,
            data: &'a str,
        ) -> Result<UserAgentCookie<'a>, ParseCookieError> {
            Ok(UserAgentCookie::new(
                self.key
                    .as_str(data)
                    .map_err(ParseCookieError::from_internal_error)?,
                self.value
                    .as_str(data)
                    .map_err(ParseCookieError::from_internal_error)?,
            ))
        }
    }
}

mod nonterminals {
    use crate::{InternalError, InternalErrorKind};

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
                None => Err(InternalError::new(
                    InternalErrorKind::NonTerminalIndexBeyondBoundaries,
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_str_all_cookie_octets, is_str_all_tokens, UserAgentCookie};

    #[test]
    fn get_name() {
        const COOKIE_KEY: &'static str = "cookie_key";
        const COOKIE_VALUE: &'static str = "cookie_value";

        let cookie = UserAgentCookie::new(COOKIE_KEY, COOKIE_VALUE);

        assert_eq!(COOKIE_KEY, cookie.get_name());
    }

    #[test]
    fn get_value() {
        const COOKIE_KEY: &'static str = "cookie_key";
        const COOKIE_VALUE: &'static str = "cookie_value";

        let cookie = UserAgentCookie::new(COOKIE_KEY, COOKIE_VALUE);

        assert_eq!(COOKIE_VALUE, cookie.get_value());
    }

    #[test]
    fn single_cookie() {
        const COOKIE_STR: &'static str = "test=1234";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("1234", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_quoted() {
        const COOKIE_STR: &'static str = "quoted_test=\"quotedval\"";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("quoted_test", parsed_cookie.name);
        assert_eq!("quotedval", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_with_equals_in_value() {
        const COOKIE_STR: &'static str = "test=abc=123";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("abc=123", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_before() {
        const COOKIE_STR: &'static str = " qtest=9878";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("qtest", parsed_cookie.name);
        assert_eq!("9878", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_after() {
        const COOKIE_STR: &'static str = "abcde=77766test \x09\x09    ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("abcde", parsed_cookie.name);
        assert_eq!("77766test", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_after() {
        const COOKIE_STR: &'static str = "xyzzz=test3 ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("xyzzz", parsed_cookie.name);
        assert_eq!("test3", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before_and_after() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876       ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name() {
        const COOKIE_STR: &'static str = "=nokey";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_with_ows_before() {
        const COOKIE_STR: &'static str = " =nokey";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value() {
        const COOKIE_STR: &'static str = "noval=";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value_with_ows_after() {
        const COOKIE_STR: &'static str = "noval= ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_and_val() {
        const COOKIE_STR: &'static str = "=";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_no_equals() {
        const COOKIE_STR: &'static str = "nokey";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn two_cookies() {
        const COOKIE_STR: &'static str = "test1=01234; test2=testval";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR).unwrap();
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
    fn emit_all_empty() {
        assert_eq!("", UserAgentCookie::emit_all(vec![]).unwrap());
    }

    #[test]
    fn emit_all_single() {
        assert_eq!(
            "testkey=testvalue",
            UserAgentCookie::emit_all(vec![UserAgentCookie::new("testkey", "testvalue")]).unwrap()
        );
    }

    #[test]
    fn emit_all_two() {
        assert_eq!(
            "abc=123; hello=world",
            UserAgentCookie::emit_all(vec![
                UserAgentCookie::new("abc", "123"),
                UserAgentCookie::new("hello", "world")
            ])
            .unwrap()
        );
    }

    #[test]
    fn emit_all_invalid_token() {
        assert!(
            UserAgentCookie::emit_all(vec![UserAgentCookie::new("[abc]", "123")]).is_err(),
            "EncodingError expected but result was successful."
        );
    }

    #[test]
    fn emit_all_invalid_cookie_value() {
        assert!(
            UserAgentCookie::emit_all(vec![UserAgentCookie {
                name: "abc",
                value: "\"123\""
            }])
            .is_err(),
            "EncodingError expected but result was successful."
        );
    }
}
