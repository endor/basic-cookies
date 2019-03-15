use crate::{
    EmitCookieError, EncodingError, EncodingErrorExpectedClass, ScanCharResult,
    ScanUntilCharResult, StringScanner,
};

#[cfg_attr(test, derive(Debug, PartialEq))]
enum ParseNameResult<'a> {
    Name(&'a str),
    Value(&'a str),
    None,
}

/// A cookie suitable to be sent from a user agent to a server, as described in [Section 4.2 of RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2).
///
/// # Examples
/// ```
/// use basic_cookies::UserAgentCookie;
///
/// let new_cookie_0 = UserAgentCookie::new("key0", "value0");
/// let new_cookie_1 = UserAgentCookie::new("key1", "value1");
/// let cookie_string = UserAgentCookie::emit_all(&vec![new_cookie_0, new_cookie_1]).unwrap();
/// assert_eq!("key0=value0; key1=value1", &cookie_string);
///
/// let parsed_cookies = UserAgentCookie::parse(&cookie_string);
/// assert_eq!("key0", parsed_cookies[0].get_name());
/// assert_eq!("value0", parsed_cookies[0].get_value());
/// assert_eq!("key1", parsed_cookies[1].get_name());
/// assert_eq!("value1", parsed_cookies[1].get_value());
/// ```
#[derive(Clone, Debug)]
pub struct UserAgentCookie<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> UserAgentCookie<'a> {
    /// Creates a new cookie that is suitable to be sent from a user agent to a server.
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
    /// let parsed_cookies = UserAgentCookie::parse("cookie1=value1; cookie2=value2");
    ///
    /// assert_eq!("cookie1", parsed_cookies[0].get_name());
    /// assert_eq!("value1", parsed_cookies[0].get_value());
    ///
    /// assert_eq!("cookie2", parsed_cookies[1].get_name());
    /// assert_eq!("value2", parsed_cookies[1].get_value());
    /// ```
    pub fn parse(input: &'a str) -> Vec<UserAgentCookie<'a>> {
        let mut results = Vec::new();
        let mut scanner = StringScanner::from_str(input);

        loop {
            match UserAgentCookie::parse_name(&mut scanner) {
                ParseNameResult::Name(name) => match UserAgentCookie::parse_value(&mut scanner) {
                    Some(val) => results.push(UserAgentCookie::new(name, val)),
                    None => {
                        results.push(UserAgentCookie::new(name, ""));
                        break;
                    }
                },
                ParseNameResult::Value(val) => results.push(UserAgentCookie::new("", val)),
                ParseNameResult::None => break,
            };
        }

        results
    }

    fn parse_name<'input>(scanner: &mut StringScanner<'input>) -> ParseNameResult<'input> {
        scanner.scan_whitespace_repeating();
        if scanner.is_at_end_of_string() {
            return ParseNameResult::None;
        }

        let start_idx = scanner.get_cursor();

        match scanner.scan_until_char('=') {
            ScanUntilCharResult::CharFound => {
                let end_idx = scanner.get_cursor();
                ParseNameResult::Name(scanner.substring(start_idx, end_idx))
            }
            ScanUntilCharResult::EndOfStringReached => {
                ParseNameResult::Value(scanner.substring(start_idx, scanner.get_cursor()))
            }
        }
    }

    fn parse_value<'input>(scanner: &mut StringScanner<'input>) -> Option<&'input str> {
        scanner.scan_char_once('=');
        let starts_with_dquote = match scanner.scan_char_once('"') {
            ScanCharResult::CharFound(_) => true,
            _ => false,
        };

        let start_idx = scanner.get_cursor();

        match if starts_with_dquote {
            scanner.scan_until_char('"')
        } else {
            scanner.scan_until_char_or_whitespace(';')
        } {
            ScanUntilCharResult::CharFound => {
                let end_idx = scanner.get_cursor();

                if starts_with_dquote {
                    scanner.scan_char_once('"');
                }

                scanner.scan_char_once(';');
                Some(scanner.substring(start_idx, end_idx))
            }
            ScanUntilCharResult::EndOfStringReached => {
                if scanner.get_cursor() > start_idx {
                    Some(scanner.substring(start_idx, scanner.get_cursor()))
                } else {
                    None
                }
            }
        }
    }

    /// Gets the name of the cookie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let parsed_cookies = UserAgentCookie::parse("name=value");
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
    /// let parsed_cookies = UserAgentCookie::parse("name=value");
    /// assert_eq!("value", parsed_cookies[0].get_value());
    /// ```
    pub fn get_value(&self) -> &'a str {
        self.value
    }
}

impl<'b, 'a: 'b> UserAgentCookie<'a> {
    /// Emits an [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) compliant cookie string that comprised of
    /// this cookie only and is suitable to be sent from a user agent to a server.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let cookie = UserAgentCookie::new("key", "value");
    /// let cookie_string = cookie.emit().unwrap();
    /// assert_eq!("key=value", cookie_string);
    /// ```
    pub fn emit(&'b self) -> Result<String, EmitCookieError<'a>> {
        UserAgentCookie::emit_all(&[self.clone()])
    }

    /// Emits an [RFC 6265](https://tools.ietf.org/html/rfc6265.html#section-4.2.1) compliant cookie string that is comprised of
    /// multiple cookies and is suitable to be sent from a user agent to a server.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_cookies::UserAgentCookie;
    ///
    /// let cookie_0 = UserAgentCookie::new("key0", "value0");
    /// let cookie_1 = UserAgentCookie::new("key1", "value1");
    /// let cookie_string = UserAgentCookie::emit_all(&vec![cookie_0, cookie_1]).unwrap();
    /// assert_eq!("key0=value0; key1=value1", cookie_string);
    /// ```
    pub fn emit_all<T: IntoIterator<Item = &'b UserAgentCookie<'a>>>(
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
    val.chars().all(is_token_char)
}

fn is_str_all_cookie_octets(val: &str) -> bool {
    val.chars().all(is_cookie_octet)
}

fn is_token_char(c: char) -> bool {
    match c {
        '\x21'
        | '\x23'...'\x27'
        | '\x2a'
        | '\x2b'
        | '\x2d'
        | '\x2e'
        | '\x30'...'\x39'
        | '\x41'...'\x5a'
        | '\x5e'...'\x7a'
        | '\x7c'
        | '\x7e' => true,
        _ => false,
    }
}

pub fn is_cookie_octet(c: char) -> bool {
    match c {
        '\x21'
        | '\x23'...'\x27'
        | '\x2a'
        | '\x2b'
        | '\x2d'
        | '\x2e'
        | '\x30'...'\x39'
        | '\x41'...'\x5a'
        | '\x5e'...'\x7a'
        | '\x7c'
        | '\x7e'
        | '\x28'
        | '\x29'
        | '\x2f'
        | '\x3a'
        | '\x3c'
        | '\x3e'...'\x40'
        | '\x5b'
        | '\x5d'
        | '\x7b'
        | '\x7d' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{is_str_all_cookie_octets, is_str_all_tokens, ParseNameResult, UserAgentCookie};
    use crate::StringScanner;

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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("1234", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_quoted() {
        const COOKIE_STR: &'static str = "quoted_test=\"quotedval\"";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("quoted_test", parsed_cookie.name);
        assert_eq!("quotedval", parsed_cookie.value);
    }

    // technically this is not compatible with RFC 6265, but it is
    // compatible with the earlier RFC 2109 and it's somewhat common,
    // not sure if this is an oversight or not
    #[test]
    fn single_cookie_quoted_with_space() {
        const COOKIE_STR: &'static str = "quoted_test=\"quoted val\"";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("quoted_test", parsed_cookie.name);
        assert_eq!("quoted val", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_with_equals_in_value() {
        const COOKIE_STR: &'static str = "test=abc=123";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("test", parsed_cookie.name);
        assert_eq!("abc=123", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_before() {
        const COOKIE_STR: &'static str = " qtest=9878";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("qtest", parsed_cookie.name);
        assert_eq!("9878", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_after() {
        const COOKIE_STR: &'static str = "abcde=77766test \x09\x09    ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("abcde", parsed_cookie.name);
        assert_eq!("77766test", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_with_single_space_after() {
        const COOKIE_STR: &'static str = "xyzzz=test3 ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("xyzzz", parsed_cookie.name);
        assert_eq!("test3", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_ows_before_and_after() {
        const COOKIE_STR: &'static str = " \x09 ztest=9876       ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("ztest", parsed_cookie.name);
        assert_eq!("9876", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name() {
        const COOKIE_STR: &'static str = "=nokey";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_with_ows_before() {
        const COOKIE_STR: &'static str = " =nokey";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value() {
        const COOKIE_STR: &'static str = "noval=";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_value_with_ows_after() {
        const COOKIE_STR: &'static str = "noval= ";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("noval", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_and_val() {
        const COOKIE_STR: &'static str = "=";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("", parsed_cookie.value);
    }

    #[test]
    fn single_cookie_empty_name_no_equals() {
        const COOKIE_STR: &'static str = "nokey";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
        assert_eq!(1, parsed_cookies.len());

        let parsed_cookie = &parsed_cookies[0];
        assert_eq!("", parsed_cookie.name);
        assert_eq!("nokey", parsed_cookie.value);
    }

    #[test]
    fn two_cookies() {
        const COOKIE_STR: &'static str = "test1=01234; test2=testval";
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
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
        let parsed_cookies = UserAgentCookie::parse(COOKIE_STR);
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
    fn parse_name_single() {
        let mut scanner = StringScanner::from_str("name=value");
        let result = UserAgentCookie::parse_name(&mut scanner);
        assert_eq!(ParseNameResult::Name("name"), result);
    }

    #[test]
    fn parse_name_multiple() {
        let mut scanner = StringScanner::from_str("name0=value0; name1=value1");
        let result = UserAgentCookie::parse_name(&mut scanner);
        assert_eq!(ParseNameResult::Name("name0"), result);
    }

    #[test]
    fn parse_name_value_only() {
        let mut scanner = StringScanner::from_str("value0");
        let result = UserAgentCookie::parse_name(&mut scanner);
        assert_eq!(ParseNameResult::Value("value0"), result);
    }

    #[test]
    fn parse_name_empty_name() {
        let mut scanner = StringScanner::from_str("=value");
        let result = UserAgentCookie::parse_name(&mut scanner);
        assert_eq!(ParseNameResult::Name(""), result);
    }

    #[test]
    fn parse_name_empty_str() {
        let mut scanner = StringScanner::from_str("");
        let result = UserAgentCookie::parse_name(&mut scanner);
        assert_eq!(ParseNameResult::None, result);
    }

    #[test]
    fn parse_value_single_with_eq() {
        let mut scanner = StringScanner::from_str("=value");
        let result = UserAgentCookie::parse_value(&mut scanner);
        assert_eq!(Some("value"), result);
    }

    #[test]
    fn parse_value_single_without_eq() {
        let mut scanner = StringScanner::from_str("=value");
        let result = UserAgentCookie::parse_value(&mut scanner);
        assert_eq!(Some("value"), result);
    }

    #[test]
    fn parse_value_multiple() {
        let mut scanner = StringScanner::from_str("=value0; name1=value1; name2=value2");
        let result = UserAgentCookie::parse_value(&mut scanner);
        assert_eq!(Some("value0"), result);
    }

    #[test]
    fn parse_value_empty_str() {
        let mut scanner = StringScanner::from_str("");
        let result = UserAgentCookie::parse_value(&mut scanner);
        assert_eq!(None, result);
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
    fn emit_valid() {
        assert_eq!(
            "z01=x987",
            UserAgentCookie::new("z01", "x987").emit().unwrap()
        );
    }

    #[test]
    fn emit_invalid_token() {
        assert!(
            UserAgentCookie::new("[abc]", "123").emit().is_err(),
            "EncodingError expected but result was successful."
        );
    }

    #[test]
    fn emit_invalid_cookie_value() {
        assert!(
            UserAgentCookie::new("test", "\"123\"").emit().is_err(),
            "EncodingError expected but result was successful."
        );
    }

    #[test]
    fn emit_all_empty() {
        assert_eq!("", UserAgentCookie::emit_all(&vec![]).unwrap());
    }

    #[test]
    fn emit_all_single() {
        assert_eq!(
            "testkey=testvalue",
            UserAgentCookie::emit_all(&vec![UserAgentCookie::new("testkey", "testvalue")]).unwrap()
        );
    }

    #[test]
    fn emit_all_two() {
        assert_eq!(
            "abc=123; hello=world",
            UserAgentCookie::emit_all(&vec![
                UserAgentCookie::new("abc", "123"),
                UserAgentCookie::new("hello", "world")
            ])
            .unwrap()
        );
    }

    #[test]
    fn emit_all_invalid_token() {
        assert!(
            UserAgentCookie::emit_all(&vec![UserAgentCookie::new("[abc]", "123")]).is_err(),
            "EncodingError expected but result was successful."
        );
    }

    #[test]
    fn emit_all_invalid_cookie_value() {
        assert!(
            UserAgentCookie::emit_all(&vec![UserAgentCookie {
                name: "abc",
                value: "\"123\""
            }])
            .is_err(),
            "EncodingError expected but result was successful."
        );
    }

    #[cfg(test)]
    mod is_token_char {
        use super::super::is_token_char;

        #[test]
        fn x00() {
            assert_eq!(false, is_token_char('\x00'));
        }

        #[test]
        fn x21() {
            assert_eq!(true, is_token_char('\x21'));
        }

        #[test]
        fn x22() {
            assert_eq!(false, is_token_char('\x22'));
        }

        #[test]
        fn x23() {
            assert_eq!(true, is_token_char('\x23'));
        }

        #[test]
        fn x28() {
            assert_eq!(false, is_token_char('\x28'));
        }

        #[test]
        fn x3d() {
            assert_eq!(false, is_token_char('\x3d'));
        }

        #[test]
        fn x3e() {
            assert_eq!(false, is_token_char('\x3e'));
        }

        #[test]
        fn x7f() {
            assert_eq!(false, is_token_char('\x7f'));
        }
    }

    #[cfg(test)]
    mod is_cookie_octet {
        use super::super::is_cookie_octet;

        #[test]
        fn x00() {
            assert_eq!(false, is_cookie_octet('\x00'));
        }

        #[test]
        fn x21() {
            assert_eq!(true, is_cookie_octet('\x21'));
        }

        #[test]
        fn x22() {
            assert_eq!(false, is_cookie_octet('\x22'));
        }

        #[test]
        fn x23() {
            assert_eq!(true, is_cookie_octet('\x23'));
        }

        #[test]
        fn x28() {
            assert_eq!(true, is_cookie_octet('\x28'));
        }

        #[test]
        fn x3d() {
            assert_eq!(false, is_cookie_octet('\x3d'));
        }

        #[test]
        fn x3e() {
            assert_eq!(true, is_cookie_octet('\x3e'));
        }

        #[test]
        fn x7f() {
            assert_eq!(false, is_cookie_octet('\x7f'));
        }
    }
}

#[cfg(all(feature = "benchmarks", test))]
mod benchmarks {
    use super::UserAgentCookie;
    use crate::test::Bencher;

    #[bench]
    fn bench_parse_single_cookie(b: &mut Bencher) {
        b.iter(|| UserAgentCookie::parse("test=1234"))
    }

    #[bench]
    fn bench_parse_single_cookie_quoted(b: &mut Bencher) {
        b.iter(|| UserAgentCookie::parse("test=\"1234\""))
    }

    #[bench]
    fn bench_parse_multiple_cookies(b: &mut Bencher) {
        b.iter(|| {
            UserAgentCookie::parse("test0=1234; test1=abcde; test2=xxxxx; test3=\"987654321\"")
        })
    }

    #[bench]
    fn emit(b: &mut Bencher) {
        b.iter(|| UserAgentCookie::new("test", "1234").emit())
    }

    #[bench]
    fn emit_all_single_cookie(b: &mut Bencher) {
        b.iter(|| {
            UserAgentCookie::emit_all(&vec![UserAgentCookie {
                name: "test0",
                value: "\"123\"",
            }])
        })
    }

    #[bench]
    fn emit_all_multiple_cookies(b: &mut Bencher) {
        b.iter(|| {
            UserAgentCookie::emit_all(&vec![
                UserAgentCookie {
                    name: "test0",
                    value: "\"123\"",
                },
                UserAgentCookie {
                    name: "test1",
                    value: "abcde",
                },
                UserAgentCookie {
                    name: "test2",
                    value: "testvalue",
                },
            ])
        })
    }
}
