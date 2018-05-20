use std::fmt::{self, Display};
use std::result::Result as StdResult;

use failure::{Context, Fail, Backtrace};

#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Invalid command")]
    InvalidCommand,

    #[fail(display = "Unknown topic")]
    UnknownTopic,
}

pub type Result<T> = StdResult<T, Error>;

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}
