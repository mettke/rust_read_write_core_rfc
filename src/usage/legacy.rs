use crate::lib::read::{Error, Read};
use crate::lib::write::Write;
use std::fmt;

pub struct TypeImplementingRead {
    data: &'static [u8],
    i: usize,
}

impl Default for TypeImplementingRead {
    fn default() -> Self {
        Self {
            data: &[65, 66, 67],
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

#[derive(Debug)]
pub struct TypeImplementingWrite {
    data: [u8; 3],
    i: usize,
}

impl Default for TypeImplementingWrite {
    fn default() -> Self {
        Self {
            data: [65, 66, 67],
            i: 0,
        }
    }
}

impl Write for TypeImplementingWrite {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let mut written = 0;
        for (des, src) in self.data.iter_mut().skip(self.i).zip(buf) {
            *des = *src;
            written += 1;
        }
        self.i += written;
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub fn fun_req_write<W: Write + fmt::Debug>(mut writer: W) {
    writer.write_all(&[0, 1, 2]).expect("Unable to write data");
    println!("{:?}", writer);
}
