use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    inner: Box<ErrorImpl>,
}

struct ErrorImpl {
    code: ErrorCode,
    line: usize,
    column: usize,
}

// TODO: Remove once they are constructed
#[allow(dead_code)]
pub(crate) enum ErrorCode {
    // Generic error built from a message or different error
    Message(String),
    // Wrap inner I/O errors
    Io(io::Error),
    Eof,
    Syntax,
    ExpectedTopLevelObject,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapEquals,
    ExpectedMapEnd,
    TrailingCharacters,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Message(msg) => f.write_str(msg),
            ErrorCode::Io(err) => fmt::Display::fmt(err, f),
            ErrorCode::Eof => f.write_str("unexpected end of input"),
            ErrorCode::Syntax => f.write_str("syntax error"),
            ErrorCode::ExpectedTopLevelObject => f.write_str("expected object at the top level"),
            ErrorCode::ExpectedBoolean => f.write_str("expected a boolean value"),
            ErrorCode::ExpectedInteger => f.write_str("expected an integer value"),
            ErrorCode::ExpectedString => f.write_str("expected a string value"),
            ErrorCode::ExpectedNull => f.write_str("expected null"),
            ErrorCode::ExpectedArray => f.write_str("expected an array value"),
            ErrorCode::ExpectedArrayEnd => f.write_str("expected an array end delimiter"),
            ErrorCode::ExpectedMap => f.write_str("expected an object value"),
            ErrorCode::ExpectedMapEquals => f.write_str("expected a '=' between key and value"),
            ErrorCode::ExpectedMapEnd => f.write_str("expected an object end delimiter"),
            ErrorCode::TrailingCharacters => f.write_str("unexpected trailing characters"),
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
            "Error({:?}, line: {}, column: {})",
            self.inner.code.to_string(),
            self.inner.line,
            self.inner.column
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
        });
        Self { inner }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn new(code: ErrorCode, line: usize, column: usize) -> Self {
        Self {
            inner: Box::new(ErrorImpl { code, line, column }),
        }
    }
}
