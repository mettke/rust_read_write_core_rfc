use crate::lib::alloc::ReadAlloc;
use crate::lib::core::{
    FormatterError, InvalidUtf8, OpRes, ReadCore, UnexpectedEndOfFile, WriteCore,
};
use core::fmt::Debug;
use core::num::NonZeroUsize;

#[derive(Debug)]
pub enum TypeImplementingCoreError {
    InvalidUtf8,
    UnexpectedEof,
    FormatterError,
}

impl From<InvalidUtf8> for TypeImplementingCoreError {
    fn from(_: InvalidUtf8) -> Self {
        Self::InvalidUtf8
    }
}

impl From<UnexpectedEndOfFile> for TypeImplementingCoreError {
    fn from(_: UnexpectedEndOfFile) -> Self {
        Self::UnexpectedEof
    }
}

impl From<FormatterError> for TypeImplementingCoreError {
    fn from(_: FormatterError) -> Self {
        Self::FormatterError
    }
}

pub struct TypeImplementingReadCore {
    data: &'static [u8],
    i: usize,
}

impl Default for TypeImplementingReadCore {
    fn default() -> Self {
        Self {
            data: &[65, 66, 67],
            i: 0,
        }
    }
}

impl ReadCore for TypeImplementingReadCore {
    type Err = TypeImplementingCoreError;

    fn read(&mut self, buf: &mut [u8]) -> Result<OpRes, Self::Err> {
        let mut written = 0;
        for (des, src) in buf.iter_mut().zip(self.data.iter().skip(self.i)) {
            *des = *src;
            written += 1;
        }
        self.i += written;
        Ok(NonZeroUsize::new(written).map_or(OpRes::Eof, |v| {
            if v.get() == buf.len() {
                OpRes::Completly(v)
            } else {
                OpRes::Partial(v)
            }
        }))
    }
}

pub fn fun_req_read_ext<
    E: Debug + From<InvalidUtf8> + From<UnexpectedEndOfFile>,
    R: ReadAlloc + ReadCore<Err = E>,
>(
    mut reader: R,
) {
    let mut data = String::new();
    reader
        .read_to_string(&mut data)
        .expect("Unable to read data");
    println!("{}", data);
}

#[derive(Debug)]
pub struct TypeImplementingWriteCore {
    data: [u8; 3],
    i: usize,
}

impl Default for TypeImplementingWriteCore {
    fn default() -> Self {
        Self {
            data: [65, 66, 67],
            i: 0,
        }
    }
}

impl WriteCore for TypeImplementingWriteCore {
    type Err = TypeImplementingCoreError;

    fn write(&mut self, buf: &[u8]) -> Result<OpRes, Self::Err> {
        let mut written = 0;
        for (des, src) in self.data.iter_mut().skip(self.i).zip(buf) {
            *des = *src;
            written += 1;
        }
        self.i += written;
        Ok(NonZeroUsize::new(written).map_or(OpRes::Eof, |v| {
            if v.get() == buf.len() {
                OpRes::Completly(v)
            } else {
                OpRes::Partial(v)
            }
        }))
    }

    fn flush(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }
}

pub fn fun_req_write<
    E: Debug + From<FormatterError> + From<UnexpectedEndOfFile>,
    W: Debug + WriteCore<Err = E>,
>(
    mut writer: W,
) {
    writer.write_all(&[0, 1, 2]).expect("Unable to write data");
    println!("{:?}", writer);
}
