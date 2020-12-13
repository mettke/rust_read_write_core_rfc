#![allow(unused_mut, unused_variables)]

use crate::rust::alloc::ReadExt;
use crate::rust::core::ReadCore;
use std::io::{Bytes, Chain, Error, IoSliceMut, Take};

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn is_read_vectored(&self) -> bool {
        // Implementation from std::io::Read
        unimplemented!();
    }

    // Initializer return type is not public api, thus elided here
    unsafe fn initializer(&self) {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn chain<R: Read>(self, next: R) -> Chain<Self, R>
    where
        Self: Sized,
    {
        // Implementation from std::io::Read
        unimplemented!();
    }

    fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        // Implementation from std::io::Read
        unimplemented!();
    }
}

impl<T: ?Sized> ReadCore for T
where
    T: Read,
{
    type Error = Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Read::read(self, buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        Read::read_exact(self, buf)
    }
}

impl<T: ?Sized> ReadExt for T
where
    T: Read,
{
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error> {
        Read::read_to_end(self, buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Self::Error> {
        Read::read_to_string(self, buf)
    }
}
