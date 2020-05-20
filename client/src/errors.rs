use std::{error::Error, fmt, io};
use url;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ErrorKind {
  ClientProtocolError,
  InvalidClientConfig,
  IoError,
  InvalidSchemeError,
  ServerProtocolError,
  TypeError,
}

#[derive(Debug)]
enum ErrorRepr {
  WithDescription(ErrorKind, &'static str),
  WithDescriptionAndDetail(ErrorKind, &'static str, String),
  IoError(io::Error),
  UrlParseError(url::ParseError),
}

#[derive(Debug)]
pub struct NatsClientError {
  repr: ErrorRepr,
}

impl Error for NatsClientError {
  fn description(&self) -> &str {
    match self.repr {
      ErrorRepr::WithDescription(_, description)
      | ErrorRepr::WithDescriptionAndDetail(_, description, _) => description,
      ErrorRepr::IoError(ref e) => e.description(),
      ErrorRepr::UrlParseError(ref e) => e.description(),
    }
  }

  fn cause(&self) -> Option<&dyn Error> {
    match self.repr {
      ErrorRepr::IoError(ref e) => Some(e as &dyn Error),
      _ => None,
    }
  }
}

impl fmt::Display for NatsClientError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match self.repr {
      ErrorRepr::WithDescription(_, description) => description.fmt(f),
      ErrorRepr::WithDescriptionAndDetail(_, description, ref detail) => {
        description.fmt(f)?;
        f.write_str(": ")?;
        detail.fmt(f)
      }
      ErrorRepr::IoError(ref e) => e.fmt(f),
      ErrorRepr::UrlParseError(ref e) => e.fmt(f),
    }
  }
}

impl From<(ErrorKind, &'static str)> for NatsClientError {
  fn from((kind, description): (ErrorKind, &'static str)) -> Self {
    NatsClientError {
      repr: ErrorRepr::WithDescription(kind, description),
    }
  }
}

impl From<(ErrorKind, &'static str, String)> for NatsClientError {
  fn from((kind, description, detail): (ErrorKind, &'static str, String)) -> Self {
    NatsClientError {
      repr: ErrorRepr::WithDescriptionAndDetail(kind, description, detail),
    }
  }
}

impl From<(io::Error)> for NatsClientError {
  fn from(e: io::Error) -> Self {
    NatsClientError {
      repr: ErrorRepr::IoError(e),
    }
  }
}

impl From<(url::ParseError)> for NatsClientError {
  fn from(e: url::ParseError) -> Self {
    NatsClientError {
      repr: ErrorRepr::UrlParseError(e),
    }
  }
}
