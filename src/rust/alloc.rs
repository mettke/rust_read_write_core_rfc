#![allow(unused_variables)]

use crate::rust::core::ReadCore;
use alloc::string::String;
use alloc::vec::Vec;

pub trait ReadExt: ReadCore {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error>;

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Self::Error>;
}

impl<T: ?Sized> ReadExt for T
where
    T: ReadCore,
{
    default fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }

    default fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Self::Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }
}
