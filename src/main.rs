#![feature(specialization)]

extern crate alloc;

mod rust {
    pub mod alloc;
    pub mod core;
    pub mod lib;
}

mod usage {
    pub mod legacy;
    pub mod new;
}

fn main() {
    read_to_readcore();
    readcore_to_read();
}

fn read_to_readcore() {
    let ty = usage::legacy::TypeImplementingRead::default();
    usage::new::fun_req_read_ext(ty);
}

fn readcore_to_read() {
    let ty = usage::new::TypeImplementingReadCore::default();
    let comp_layer = LegacyRead(ty);
    usage::legacy::fun_req_read(comp_layer);
}

use crate::rust::alloc::ReadExt;
use crate::rust::core::ReadCore;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind};

#[derive(Debug)]
struct LegacyError<E: fmt::Debug>(E);

impl<E: fmt::Debug> StdError for LegacyError<E> {}

impl<E: fmt::Debug> fmt::Display for LegacyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to read from Reader. Error: {:?}", self.0)
    }
}

struct LegacyRead<R: ReadExt>(R);

impl<Error, Reader> crate::rust::lib::Read for LegacyRead<Reader>
where
    Error: fmt::Debug + Send + Sync + 'static,
    Reader: ReadExt + ReadCore<Error = Error>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.0
            .read(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, IoError> {
        self.0
            .read_to_end(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, IoError> {
        self.0
            .read_to_string(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), IoError> {
        self.0
            .read_exact(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }
}
