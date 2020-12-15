use crate::lib::alloc::ReadAlloc;
use crate::lib::core::{InvalidUtf8, OpRes, ReadCore, UnexpectedEndOfFile};
use core::fmt::Debug;
use core::num::NonZeroUsize;

#[derive(Debug)]
pub enum TypeImplementingReadCoreError {
    InvalidUtf8,
    UnexpectedEof,
}

impl From<InvalidUtf8> for TypeImplementingReadCoreError {
    fn from(_: InvalidUtf8) -> Self {
        Self::InvalidUtf8
    }
}

impl From<UnexpectedEndOfFile> for TypeImplementingReadCoreError {
    fn from(_: UnexpectedEndOfFile) -> Self {
        Self::UnexpectedEof
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
    type Err = TypeImplementingReadCoreError;

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
