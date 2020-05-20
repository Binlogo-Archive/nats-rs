use std::io::{Read, Result, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum Stream {
  Tcp(TcpStream),
}

impl Stream {
  pub fn try_clone(&self) -> Result<Stream> {
    match *self {
      Stream::Tcp(ref s) => Ok(Stream::Tcp(s.try_clone()?)),
    }
  }

  pub fn as_tcp(&self) -> Result<TcpStream> {
    match *self {
      Stream::Tcp(ref s) => s.try_clone(),
    }
  }
}

impl Read for Stream {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    match *self {
      Stream::Tcp(ref mut s) => s.read(buf),
    }
  }
}

impl Write for Stream {
  fn write(&mut self, buf: &[u8]) -> Result<usize> {
    match *self {
      Stream::Tcp(ref mut s) => s.write(buf),
    }
  }

  fn flush(&mut self) -> Result<()> {
    match *self {
      Stream::Tcp(ref mut s) => s.flush(),
    }
  }
}
