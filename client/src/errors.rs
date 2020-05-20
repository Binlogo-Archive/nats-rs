use std::io;
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
