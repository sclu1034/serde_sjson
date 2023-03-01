use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{char, digit1, none_of, not_line_ending, one_of};
use nom::combinator::{cut, eof, map, map_res, opt, recognize, value};
use nom::multi::many1_count;
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{IResult, Slice};
use nom_locate::LocatedSpan;

pub(crate) type Span<'a> = LocatedSpan<&'a str>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    ArrayEnd,
    ArrayStart,
    Boolean(bool),
    Eof,
    Equals,
    Float(f64),
    Integer(i64),
    Null,
    ObjectEnd,
    ObjectStart,
    Separator,
    String(String),
}

fn horizontal_whitespace(input: Span) -> IResult<Span, char> {
    one_of(" \t")(input)
}

fn whitespace(input: Span) -> IResult<Span, char> {
    one_of(" \n\r\t")(input)
}

fn null(input: Span) -> IResult<Span, ()> {
    value((), tag("null"))(input)
}

fn separator(input: Span) -> IResult<Span, &str> {
    map(alt((tag(","), tag("\n"))), |val: Span| *val.fragment())(input)
}

fn bool(input: Span) -> IResult<Span, bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(input)
}

fn integer(input: Span) -> IResult<Span, i64> {
    map_res(recognize(tuple((opt(char('-')), digit1))), |val: Span| {
        val.fragment().parse::<i64>()
    })(input)
}

fn float(input: Span) -> IResult<Span, f64> {
    double(input)
}

fn identifier(input: Span) -> IResult<Span, &str> {
    map(recognize(many1_count(none_of("\" \t\n=:"))), |val: Span| {
        *val.fragment()
    })(input)
}

fn string_content(input: Span) -> IResult<Span, &str> {
    let buf = input.fragment();
    let mut escaped = false;
    let mut i = 0;

    for (j, ch) in buf.char_indices() {
        i = j;
        match ch {
            '\\' if !escaped => {
                escaped = true;
            }
            '\n' if !escaped => {
                let err = nom::error::Error {
                    input: input.slice(j..),
                    code: nom::error::ErrorKind::Char,
                };
                return Err(nom::Err::Error(err));
            }
            '"' if !escaped => {
                return Ok((input.slice(j..), &buf[0..j]));
            }
            _ => escaped = false,
        }
    }

    let err = nom::error::Error {
        input: input.slice((i + 1)..),
        code: nom::error::ErrorKind::Char,
    };
    Err(nom::Err::Failure(err))
}

fn delimited_string(input: Span) -> IResult<Span, &str> {
    preceded(char('"'), cut(terminated(string_content, char('"'))))(input)
}

fn string(input: Span) -> IResult<Span, &str> {
    alt((identifier, delimited_string))(input)
}

fn line_comment(input: Span) -> IResult<Span, &str> {
    map(
        preceded(tag("//"), alt((not_line_ending, eof))),
        |val: Span| *val.fragment(),
    )(input)
}

fn block_comment(input: Span) -> IResult<Span, &str> {
    map(
        delimited(tag("/*"), take_until("*/"), tag("*/")),
        |val: Span| *val.fragment(),
    )(input)
}

fn comment(input: Span) -> IResult<Span, &str> {
    alt((line_comment, block_comment))(input)
}

fn optional(input: Span) -> IResult<Span, ()> {
    let whitespace = value((), whitespace);
    let comment = value((), comment);
    let empty = value((), tag(""));
    let content = value((), many1_count(alt((whitespace, comment))));

    alt((content, empty))(input)
}

pub(crate) fn parse_next_token(input: Span) -> IResult<Span, Token> {
    preceded(
        opt(optional),
        alt((
            // Order is important here.
            // Certain valid strings like "null", "true" or "false" need to be
            // matched to their special value.
            // Integer-like numbers need to be matched to that, but are valid floats, too.
            value(Token::Eof, eof),
            value(Token::Separator, separator),
            value(Token::ObjectStart, tag("{")),
            value(Token::ObjectEnd, tag("}")),
            value(Token::ArrayStart, tag("[")),
            value(Token::ArrayEnd, tag("]")),
            value(Token::Equals, tag("=")),
            value(Token::Null, null),
            map(bool, Token::Boolean),
            map(integer, Token::Integer),
            map(float, Token::Float),
            map(string, |val| Token::String(val.to_string())),
        )),
    )(input)
}

pub(crate) fn parse_trailing_characters(input: Span) -> IResult<Span, ()> {
    value((), optional)(input)
}

pub(crate) fn parse_null(input: Span) -> IResult<Span, Token> {
    preceded(optional, value(Token::Null, null))(input)
}

pub(crate) fn parse_separator(input: Span) -> IResult<Span, Token> {
    preceded(
        opt(horizontal_whitespace),
        value(Token::Separator, separator),
    )(input)
}

pub(crate) fn parse_bool(input: Span) -> IResult<Span, Token> {
    preceded(optional, map(bool, Token::Boolean))(input)
}

pub(crate) fn parse_integer(input: Span) -> IResult<Span, Token> {
    preceded(optional, map(integer, Token::Integer))(input)
}

pub(crate) fn parse_float(input: Span) -> IResult<Span, Token> {
    preceded(optional, map(float, Token::Float))(input)
}

pub(crate) fn parse_identifier(input: Span) -> IResult<Span, Token> {
    preceded(
        optional,
        map(identifier, |val| Token::String(val.to_string())),
    )(input)
}

pub(crate) fn parse_string(input: Span) -> IResult<Span, Token> {
    preceded(optional, map(string, |val| Token::String(val.to_string())))(input)
}

#[cfg(test)]
mod test {
    use nom::error::{Error, ErrorKind};
    use nom::Err;

    use super::*;

    macro_rules! assert_ok {
        ($input:expr, $parser:ident, $remain:expr, $output:expr) => {{
            let res = super::$parser(Span::from($input));
            assert_eq!(
                res.map(|(span, res)| { (*span, res) }),
                Ok(($remain, $output))
            );
        }};
    }

    macro_rules! assert_err {
        ($input:expr, $parser:ident, $kind:expr) => {{
            {
                let input = Span::from($input);
                assert_eq!(
                    super::$parser(input),
                    Err(Err::Error(Error::new(input, $kind)))
                );
            }
        }};
    }

    fn check_parse_result<S: AsRef<str>, T: AsRef<[Token]>>(input: S, tokens: T) {
        let tokens = tokens.as_ref();
        let mut remaining = Span::from(input.as_ref());
        let mut i = 0;

        loop {
            if remaining.fragment().is_empty() {
                break;
            }

            let (span, token) =
                super::parse_next_token(remaining).expect("failed to parse next token");

            assert_eq!(Some(&token), tokens.get(i));

            remaining = span;
            i = i + 1;
        }

        assert_eq!(
            tokens.len(),
            i,
            "tokens to check against were not exhausted"
        );
    }

    #[test]
    fn parse_optional() {
        assert_ok!("\n", whitespace, "", '\n');
        assert_ok!("\t", whitespace, "", '\t');
        assert_ok!("  ", whitespace, " ", ' ');
        assert_ok!("/* foo bar */", comment, "", " foo bar ");
        assert_ok!("// foo", comment, "", " foo");
        assert_ok!("// foo\n", comment, "\n", " foo");

        assert_ok!("", optional, "", ());
        assert_ok!("\t\n", optional, "", ());
        assert_ok!("\n\t", optional, "", ());
        assert_ok!("// foo", optional, "", ());
        assert_ok!("\n\t// foo\n\t/* foo\n\tbar */\n", optional, "", ());
    }

    #[test]
    fn parse_integer() {
        assert_ok!("3", integer, "", 3);
        assert_ok!("12345", integer, "", 12345);
        assert_ok!("-12345", integer, "", -12345);
        assert_ok!("12345   ", integer, "   ", 12345);

        assert_err!("   12345", integer, ErrorKind::Digit);

        assert_ok!("    12345", parse_integer, "", Token::Integer(12345));
        assert_ok!("\n12345", parse_integer, "", Token::Integer(12345));
        assert_ok!("\t12345", parse_integer, "", Token::Integer(12345));
    }

    #[test]
    fn parse_float() {
        assert_ok!("3", float, "", 3.0);
        assert_ok!("3.0", float, "", 3.0);
        assert_ok!("3.1415", float, "", 3.1415);
        assert_ok!("-123.456789", float, "", -123.456789);
        assert_err!("   1.23", float, ErrorKind::Float);
        assert_ok!("1.23   ", float, "   ", 1.23);
    }

    #[test]
    fn parse_raw_string() {
        assert_ok!("foo", identifier, "", "foo");
        assert_ok!("foo123", identifier, "", "foo123");
        assert_ok!("foo_bar", identifier, "", "foo_bar");
        assert_ok!("_foo", identifier, "", "_foo");
        assert_ok!("foo bar", identifier, " bar", "foo");
        assert_ok!("123", identifier, "", "123");
        assert_ok!("1foo", identifier, "", "1foo");
        assert_ok!("foo-bar", identifier, "", "foo-bar");
        assert_ok!("foo/bar", identifier, "", "foo/bar");
        assert_ok!("foo\"", identifier, "\"", "foo");

        assert_err!("\"foo", identifier, ErrorKind::Many1Count);
        assert_err!("\"foo\"", identifier, ErrorKind::Many1Count);
    }

    #[test]
    fn parse_delimited_string() {
        assert_ok!(r#""""#, delimited_string, "", "");
        assert_ok!(r#""foo""#, delimited_string, "", "foo");
        assert_ok!(r#""\"foo""#, delimited_string, "", r#"\"foo"#);
        assert_ok!(r#""foo bar""#, delimited_string, "", "foo bar");
        assert_ok!(r#""foo123""#, delimited_string, "", "foo123");
        assert_ok!(r#""123foo""#, delimited_string, "", "123foo");
        assert_ok!(r#""foo\"bar""#, delimited_string, "", "foo\\\"bar");
        assert_ok!(r#""foo\\bar""#, delimited_string, "", "foo\\\\bar");
        assert_ok!(r#""foo/bar""#, delimited_string, "", "foo/bar");

        assert_err!("foo\"", delimited_string, ErrorKind::Char);

        {
            let input = Span::from("\"foo");
            assert_eq!(
                delimited_string(input),
                Err(Err::Failure(Error::new(
                    unsafe { Span::new_from_raw_offset(4, 1, "", ()) },
                    ErrorKind::Char
                )))
            );
        }

        {
            let input = Span::from("\"foo\nbar\"");
            assert_eq!(
                delimited_string(input),
                Err(Err::Failure(Error::new(
                    unsafe { Span::new_from_raw_offset(4, 1, "\nbar\"", ()) },
                    ErrorKind::Char
                )))
            );
        }
    }

    #[test]
    fn parse_line_comment() {
        assert_ok!("// foo", line_comment, "", " foo");
        assert_ok!("// foo\n", line_comment, "\n", " foo");
    }

    #[test]
    fn parse_block_comment() {
        assert_ok!("/* foo */", block_comment, "", " foo ");
        assert_ok!("/*\n\tfoo\nbar\n*/", block_comment, "", "\n\tfoo\nbar\n");
    }

    // Regression test for #1 (https://git.sclu1034.dev/lucas/serde_sjson/issues/1)
    #[test]
    fn parse_dtmt_config() {
        let sjson = r#"
name = "test-mod"
description = "A dummy project to test things with"
version = "0.1.0"

packages = [
    "packages/test-mod"
]
"#;

        check_parse_result(
            sjson,
            [
                Token::String(String::from("name")),
                Token::Equals,
                Token::String(String::from("test-mod")),
                Token::String(String::from("description")),
                Token::Equals,
                Token::String(String::from("A dummy project to test things with")),
                Token::String(String::from("version")),
                Token::Equals,
                Token::String(String::from("0.1.0")),
                Token::String(String::from("packages")),
                Token::Equals,
                Token::ArrayStart,
                Token::String(String::from("packages/test-mod")),
                Token::ArrayEnd,
                Token::Eof,
            ],
        );
    }

    // Regression test for #2
    #[test]
    fn parse_windows_path() {
        let text = "C:\\Users\\public\\test.txt";
        let sjson = format!(r#""{}""#, text);
        check_parse_result(sjson, [Token::String(String::from(text))]);
    }
}
