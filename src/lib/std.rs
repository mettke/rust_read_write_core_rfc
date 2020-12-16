use super::alloc::ReadAlloc;
use super::core::{Bytes, Initializer, OpRes, ReadCore, Take, UnexpectedEndOfFile, WriteCore};
use super::read::{Error, Read};
use super::write::Write;
use core::num::NonZeroUsize;
use std::fmt;
use std::io::{ErrorKind, IoSlice, IoSliceMut};

pub trait ReadStd: ReadAlloc {
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<OpRes, Self::Err>;

    fn is_read_vectored(&self) -> bool;
}

impl<T: ?Sized> ReadStd for T
where
    T: ReadAlloc,
{
    default fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<OpRes, Self::Err> {
        default_read_vectored(|b| self.read(b), bufs)
    }

    default fn is_read_vectored(&self) -> bool {
        false
    }
}

pub(crate) fn default_read_vectored<F, E>(read: F, bufs: &mut [IoSliceMut<'_>]) -> Result<OpRes, E>
where
    F: FnOnce(&mut [u8]) -> Result<OpRes, E>,
{
    let buf = bufs
        .iter_mut()
        .find(|b| !b.is_empty())
        .map_or(&mut [][..], |b| &mut **b);
    read(buf)
}

impl<T: ?Sized> ReadCore for T
where
    T: Read,
{
    type Err = Error;

    default fn read(&mut self, buf: &mut [u8]) -> Result<OpRes, Self::Err> {
        match Read::read(self, buf) {
            Ok(0) => Ok(OpRes::Eof),
            Ok(n) if n == buf.len() => unsafe {
                Ok(OpRes::Completly(NonZeroUsize::new_unchecked(n)))
            },
            Ok(n) => unsafe { Ok(OpRes::Partial(NonZeroUsize::new_unchecked(n))) },
            Err(e) if e.kind() == ErrorKind::Interrupted => Ok(OpRes::Retry),
            Err(e) => Err(e),
        }
    }

    default unsafe fn initializer(&self) -> Initializer {
        Read::initializer(self)
    }

    default fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Err> {
        Read::read_exact(self, buf).map(|_| ())
    }

    #[must_use]
    default fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        Read::by_ref(self)
    }

    default fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Read::bytes(self)
    }

    default fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        Read::take(self, limit)
    }
}

impl<T: ?Sized> ReadAlloc for T
where
    T: Read + ReadCore<Err = Error>,
{
    default fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<(), Self::Err> {
        Read::read_to_end(self, buf).map(|_| ())
    }

    default fn read_to_string(&mut self, buf: &mut String) -> Result<(), Self::Err> {
        Read::read_to_string(self, buf).map(|_| ())
    }
}

impl<T: ?Sized> ReadStd for T
where
    T: Read + ReadCore<Err = Error>,
{
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<OpRes, Self::Err> {
        match Read::read_vectored(self, bufs) {
            Ok(0) => Ok(OpRes::Eof),
            Ok(n) => unsafe {
                let full_len = bufs.iter().map(|b| b.len()).sum();
                Ok(if n == full_len {
                    OpRes::Completly(NonZeroUsize::new_unchecked(n))
                } else {
                    OpRes::Partial(NonZeroUsize::new_unchecked(n))
                })
            },
            Err(e) if e.kind() == ErrorKind::Interrupted => Ok(OpRes::Retry),
            Err(e) => Err(e),
        }
    }

    fn is_read_vectored(&self) -> bool {
        Read::is_read_vectored(self)
    }
}

pub trait WriteStd: WriteCore {
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<OpRes, Self::Err>;

    fn is_write_vectored(&self) -> bool;

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<(), Self::Err>;
}

impl<T: ?Sized> WriteStd for T
where
    T: WriteCore,
{
    default fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<OpRes, Self::Err> {
        default_write_vectored(|b| self.write(b), bufs)
    }

    default fn is_write_vectored(&self) -> bool {
        false
    }

    default fn write_all_vectored(
        &mut self,
        mut bufs: &mut [IoSlice<'_>],
    ) -> Result<(), Self::Err> {
        bufs = IoSlice::advance(bufs, 0);
        while !bufs.is_empty() {
            match self.write_vectored(bufs)? {
                OpRes::Eof => return Err(Self::Err::from(UnexpectedEndOfFile)),
                OpRes::Retry => {}
                OpRes::Completly(n) => {
                    IoSlice::advance(bufs, n.get());
                    break;
                }
                OpRes::Partial(n) => {
                    IoSlice::advance(bufs, n.get());
                }
            }
        }
        Ok(())
    }
}

impl<T> WriteCore for T
where
    T: Write,
{
    type Err = Error;

    default fn write(&mut self, buf: &[u8]) -> Result<OpRes, Self::Err> {
        match Write::write(self, buf) {
            Ok(0) => Ok(OpRes::Eof),
            Ok(n) if n == buf.len() => unsafe {
                Ok(OpRes::Completly(NonZeroUsize::new_unchecked(n)))
            },
            Ok(n) => unsafe { Ok(OpRes::Partial(NonZeroUsize::new_unchecked(n))) },
            Err(e) if e.kind() == ErrorKind::Interrupted => Ok(OpRes::Retry),
            Err(e) => Err(e),
        }
    }

    default fn flush(&mut self) -> Result<(), Self::Err> {
        Write::flush(self)
    }

    default fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Err> {
        Write::write_all(self, buf)
    }

    default fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<(), Self::Err> {
        Write::write_fmt(self, fmt)
    }

    default fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        Write::by_ref(self)
    }
}

impl<T> WriteStd for T
where
    T: Write + WriteCore<Err = Error>,
{
    default fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<OpRes, Self::Err> {
        match Write::write_vectored(self, bufs) {
            Ok(0) => Ok(OpRes::Eof),
            Ok(n) => unsafe {
                let full_len = bufs.iter().map(|b| b.len()).sum();
                Ok(if n == full_len {
                    OpRes::Completly(NonZeroUsize::new_unchecked(n))
                } else {
                    OpRes::Partial(NonZeroUsize::new_unchecked(n))
                })
            },
            Err(e) if e.kind() == ErrorKind::Interrupted => Ok(OpRes::Retry),
            Err(e) => Err(e),
        }
    }

    default fn is_write_vectored(&self) -> bool {
        Write::is_write_vectored(self)
    }

    default fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<(), Self::Err> {
        Write::write_all_vectored(self, bufs)
    }
}

pub(crate) fn default_write_vectored<F, E>(write: F, bufs: &[IoSlice<'_>]) -> Result<OpRes, E>
where
    F: FnOnce(&[u8]) -> Result<OpRes, E>,
{
    let buf = bufs
        .iter()
        .find(|b| !b.is_empty())
        .map_or(&[][..], |b| &**b);
    write(buf)
}
