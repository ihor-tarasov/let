#[derive(Debug)]
pub enum Error {
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    Custom(Box<String>),
    IO(std::io::Error),
    FromUtf8(Box<std::string::FromUtf8Error>),
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloat(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8(Box::new(value))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Utf8(error) => write!(f, "{error}"),
            Error::ParseInt(error) => write!(f, "{error}"),
            Error::ParseFloat(error) => write!(f, "{error}"),
            Error::Custom(error) => write!(f, "{error}"),
            Error::IO(error) => write!(f, "{error}"),
            Error::FromUtf8(error) => write!(f, "{error}"),
        }
    }
}

#[macro_export]
macro_rules! raise {
    ($($arg:tt)*) => {
        Err($crate::Error::Custom(Box::new(format!($($arg)*))))
    };
}
