use crate::rust::lib::Read;
use std::io::Error;

pub struct TypeImplementingRead {
    data: &'static [u8],
    i: usize,
}

impl Default for TypeImplementingRead {
    fn default() -> Self {
        Self {
            data: &[0, 1, 2],
            i: 0,
        }
    }
}

impl Read for TypeImplementingRead {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut written = 0;
        for (des, src) in buf.iter_mut().zip(self.data.iter().skip(self.i)) {
            *des = *src;
            written += 1;
        }
        self.i += written;
        Ok(written)
    }
}

pub fn fun_req_read<R: Read>(mut reader: R) {
    let mut data = String::new();
    reader
        .read_to_string(&mut data)
        .expect("Unable to read data");
    println!("{}", data);
}
