#![feature(specialization)]
#![feature(read_initializer)]
#![feature(io_slice_advance)]
// #![no_std]
#![allow(dead_code, incomplete_features, unreachable_code)]

extern crate alloc;

mod lib {
    pub mod alloc;
    pub mod core;
    pub mod read;
    pub mod std;
    pub mod write;
}

mod usage {
    pub mod legacy;
    pub mod new;
}

fn main() {
    read_to_readcore();
    readcore_to_read();
    write_to_writecore();
    writecore_to_write();
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

fn write_to_writecore() {
    let ty = usage::legacy::TypeImplementingWrite::default();
    usage::new::fun_req_write(ty);
}

fn writecore_to_write() {
    let ty = usage::new::TypeImplementingWriteCore::default();
    let comp_layer = LegacyWrite(ty);
    usage::legacy::fun_req_write(comp_layer);
}

use crate::lib::core::{
    FormatterError, Initializer, InvalidUtf8, OpRes, ReadCore, UnexpectedEndOfFile, WriteCore,
};
use crate::lib::read::{Error as IoError, Read};
use crate::lib::std::{ReadStd, WriteStd};
use crate::lib::write::Write;
use std::error::Error as StdError;
use std::fmt;
use std::io::{ErrorKind, IoSlice, IoSliceMut};

#[derive(Debug)]
struct LegacyError<E: fmt::Debug>(E);

impl<E: fmt::Debug> StdError for LegacyError<E> {}

impl<E: fmt::Debug> fmt::Display for LegacyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to read from Reader. Error: {:?}", self.0)
    }
}

struct LegacyRead<R: ReadStd>(R);

impl<Error, Reader> Read for LegacyRead<Reader>
where
    Error: fmt::Debug + Send + Sync + 'static + From<InvalidUtf8> + From<UnexpectedEndOfFile>,
    Reader: ReadStd + ReadCore<Err = Error>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.0
            .read(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
            .and_then(|res| match res {
                OpRes::Retry => Err(IoError::new(
                    ErrorKind::Interrupted,
                    "Read was Interrupted. Try again.",
                )),
                OpRes::Partial(n) => Ok(n.get()),
                OpRes::Completly(n) => Ok(n.get()),
                OpRes::Eof => Ok(0),
            })
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, IoError> {
        self.0
            .read_vectored(bufs)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
            .and_then(|res| match res {
                OpRes::Retry => Err(IoError::new(
                    ErrorKind::Interrupted,
                    "Read was Interrupted. Try again.",
                )),
                OpRes::Partial(n) => Ok(n.get()),
                OpRes::Completly(n) => Ok(n.get()),
                OpRes::Eof => Ok(0),
            })
    }

    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        self.0.initializer()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, IoError> {
        self.0
            .read_to_end(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
            .map(|_| buf.len())
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, IoError> {
        self.0
            .read_to_string(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
            .map(|_| buf.len())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), IoError> {
        self.0
            .read_exact(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Debug)]
struct LegacyWrite<W: WriteStd>(W);

impl<Error, Writer> Write for LegacyWrite<Writer>
where
    Error: fmt::Debug + Send + Sync + 'static + From<FormatterError> + From<UnexpectedEndOfFile>,
    Writer: WriteStd + WriteCore<Err = Error>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.0
            .write(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
            .and_then(|res| match res {
                OpRes::Retry => Err(IoError::new(
                    ErrorKind::Interrupted,
                    "Read was Interrupted. Try again.",
                )),
                OpRes::Partial(n) => Ok(n.get()),
                OpRes::Completly(n) => Ok(n.get()),
                OpRes::Eof => Ok(0),
            })
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize, IoError> {
        self.0
            .write_vectored(bufs)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
            .and_then(|res| match res {
                OpRes::Retry => Err(IoError::new(
                    ErrorKind::Interrupted,
                    "Read was Interrupted. Try again.",
                )),
                OpRes::Partial(n) => Ok(n.get()),
                OpRes::Completly(n) => Ok(n.get()),
                OpRes::Eof => Ok(0),
            })
    }

    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.0
            .flush()
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), IoError> {
        self.0
            .write_all(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<(), IoError> {
        self.0
            .write_all_vectored(bufs)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<(), IoError> {
        self.0
            .write_fmt(fmt)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}
