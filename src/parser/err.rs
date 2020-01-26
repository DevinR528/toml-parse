use std::io;
use std::num::{ParseFloatError, ParseIntError};

use chrono::format::ParseError as ChronoParseError;

pub type TomlResult<T> = Result<T, ParseTomlError>;

#[derive(PartialEq)]
pub enum TomlErrorKind {
    UnexpectedToken { tkn: String, ln: usize, col: usize },
    DateError,
    NumberError,
    InternalParseError(String),
}

#[derive(PartialEq)]
pub struct ParseTomlError {
    pub(super) kind: TomlErrorKind,
    pub(super) info: String,
}

impl ParseTomlError {
    pub fn new(s: String, t_err: TomlErrorKind) -> ParseTomlError {
        ParseTomlError {
            kind: t_err,
            info: s,
        }
    }
}

impl From<io::Error> for ParseTomlError {
    fn from(e: io::Error) -> ParseTomlError {
        let msg = e.to_string();
        ParseTomlError::new(
            msg,
            TomlErrorKind::InternalParseError("? opperator returned error".to_owned()),
        )
    }
}

impl From<ParseTomlError> for io::Error {
    fn from(e: ParseTomlError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e.info)
    }
}

impl From<chrono::format::ParseError> for ParseTomlError {
    fn from(e: ChronoParseError) -> ParseTomlError {
        let msg = e.to_string();
        ParseTomlError::new(msg, TomlErrorKind::DateError)
    }
}

impl From<ParseFloatError> for ParseTomlError {
    fn from(e: ParseFloatError) -> ParseTomlError {
        let msg = e.to_string();
        ParseTomlError::new(msg, TomlErrorKind::NumberError)
    }
}

impl From<ParseIntError> for ParseTomlError {
    fn from(e: ParseIntError) -> ParseTomlError {
        let msg = e.to_string();
        ParseTomlError::new(msg, TomlErrorKind::NumberError)
    }
}

impl std::error::Error for ParseTomlError {
    fn description(&self) -> &str {
        self.info.as_str()
    }
}

impl std::fmt::Debug for ParseTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for ParseTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let span = match &self.kind {
            TomlErrorKind::InternalParseError(span) => span.into(),
            TomlErrorKind::UnexpectedToken { tkn, ln, col } => {
                format!("{} at ln {}, col {}", tkn, ln, col)
            },
            TomlErrorKind::DateError => "an invalid date-time".into(),
            TomlErrorKind::NumberError => "an invalid number".into(),
        };
        write!(f, "{}, found {}", self.info, span)
    }
}
