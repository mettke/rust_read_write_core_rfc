#![allow(unused_variables)]

pub trait ReadCore {
    type Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        // Implementation from std::io::Read
        unimplemented!()
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        // Implementation from std::io::Read
        unimplemented!();
    }
}
