use super::alloc::ReadAlloc;
use super::core::{Bytes, Initializer, OpRes, ReadCore, Take};
use super::read::{Error, Read};
use core::num::NonZeroUsize;
use std::io::{ErrorKind, IoSliceMut};

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
