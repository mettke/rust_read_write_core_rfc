use crate::rust::alloc::ReadExt;
use crate::rust::core::ReadCore;
use core::fmt::Debug;

pub struct TypeImplementingReadCore {
    data: &'static [u8],
    i: usize,
}

impl Default for TypeImplementingReadCore {
    fn default() -> Self {
        Self {
            data: &[0, 1, 2],
            i: 0,
        }
    }
}

impl ReadCore for TypeImplementingReadCore {
    type Error = &'static str;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut written = 0;
        for (des, src) in buf.iter_mut().zip(self.data.iter().skip(self.i)) {
            *des = *src;
            written += 1;
        }
        self.i += written;
        Ok(written)
    }
}

pub fn fun_req_read_ext<E: Debug, R: ReadExt + ReadCore<Error = E>>(mut reader: R) {
    let mut data = String::new();
    reader
        .read_to_string(&mut data)
        .expect("Unable to read data");
    println!("{}", data);
}
