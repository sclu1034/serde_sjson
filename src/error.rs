use std::{fmt, io};

use crate::parser::Token;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq)]
pub struct Error {
    inner: Box<ErrorImpl>,
}

#[derive(PartialEq)]
struct ErrorImpl {
    code: ErrorCode,
    line: u32,
    column: usize,
    fragment: Option<String>,
    token: Option<Token>,
}

#[derive(PartialEq)]
pub(crate) enum ErrorCode {
    // Generic error built from a message or different error
    Message(String),
    ExpectedArray,
    ExpectedArrayEnd,
    ExpectedArraySeparator,
    ExpectedBoolean,
    ExpectedEnum,
    ExpectedFloat,
    ExpectedInteger,
    ExpectedMap,
    ExpectedMapEnd,
    ExpectedMapEquals,
    ExpectedMapSeparator,
    ExpectedNull,
    ExpectedString,
    ExpectedTopLevelObject,
    ExpectedValue,
    TrailingCharacters,
    NonFiniteFloat,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Message(msg) => f.write_str(msg),
            ErrorCode::ExpectedArray => f.write_str("expected an array value"),
            ErrorCode::ExpectedArrayEnd => f.write_str("expected an array end delimiter"),
            ErrorCode::ExpectedArraySeparator => {
                f.write_str("expected comma or newline between array entries")
            }
            ErrorCode::ExpectedBoolean => f.write_str("expected a boolean value"),
            ErrorCode::ExpectedEnum => f.write_str("expected string or object"),
            ErrorCode::ExpectedFloat => f.write_str("expected floating point number"),
            ErrorCode::ExpectedInteger => f.write_str("expected an integer value"),
            ErrorCode::ExpectedMap => f.write_str("expected an object"),
            ErrorCode::ExpectedMapEnd => f.write_str("expected an object end delimiter"),
            ErrorCode::ExpectedMapEquals => f.write_str("expected a '=' between key and value"),
            ErrorCode::ExpectedMapSeparator => {
                f.write_str("expected comma or newline between object entries")
            }
            ErrorCode::ExpectedNull => f.write_str("expected null"),
            ErrorCode::ExpectedString => f.write_str("expected a string value"),
            ErrorCode::ExpectedTopLevelObject => f.write_str("expected object at the top level"),
            ErrorCode::ExpectedValue => f.write_str("expected a value"),
            ErrorCode::TrailingCharacters => f.write_str("unexpected trailing characters"),
            ErrorCode::NonFiniteFloat => f.write_str("got infinite floating point number"),
        }
    }
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            fmt::Display::fmt(&self.code, f)
        } else {
            write!(
                f,
                "{} at line {} column {}",
                self.code, self.line, self.column
            )
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error({:?}, line: {}, column: {}, fragment: {:?}, token: {:?})",
            self.inner.code.to_string(),
            self.inner.line,
            self.inner.column,
            self.inner.fragment,
            self.inner.token,
        )
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: ToString,
    {
        let inner = Box::new(ErrorImpl {
            code: ErrorCode::Message(msg.to_string()),
            line: 0,
            column: 0,
            fragment: None,
            token: None,
        });
        Self { inner }
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: ToString,
    {
        let inner = Box::new(ErrorImpl {
            code: ErrorCode::Message(msg.to_string()),
            line: 0,
            column: 0,
            fragment: None,
            token: None,
        });
        Self { inner }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn new(code: ErrorCode, line: u32, column: usize, fragment: Option<String>) -> Self {
        Self {
            inner: Box::new(ErrorImpl {
                code,
                line,
                column,
                fragment,
                token: None,
            }),
        }
    }
    pub(crate) fn with_token(
        code: ErrorCode,
        line: u32,
        column: usize,
        fragment: Option<String>,
        token: Token,
    ) -> Self {
        Self {
            inner: Box::new(ErrorImpl {
                code,
                line,
                column,
                fragment,
                token: Some(token),
            }),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::new(ErrorCode::Message(format!("{}", err)), 0, 0, None)
    }
}
