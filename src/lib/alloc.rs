use super::core::{InvalidUtf8, OpRes, ReadCore, Take, UnexpectedEndOfFile};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use core::str;

pub trait ReadAlloc: ReadCore {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<(), Self::Err>;

    fn read_to_string(&mut self, buf: &mut String) -> Result<(), Self::Err>;
}

impl<T: ?Sized> ReadAlloc for T
where
    T: ReadCore,
{
    default fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<(), Self::Err> {
        Self::read_to_end(self, buf)
    }

    default fn read_to_string(&mut self, buf: &mut String) -> Result<(), Self::Err> {
        append_to_string(buf, |b| read_to_end(self, b))
    }
}

fn read_to_end<E, R: ReadCore<Err = E> + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> Result<(), E>
where
    E: From<InvalidUtf8> + From<UnexpectedEndOfFile>,
{
    read_to_end_with_reservation(r, buf, |_| 32)
}

fn read_to_end_with_reservation<E, R, F>(
    r: &mut R,
    buf: &mut Vec<u8>,
    mut reservation_size: F,
) -> Result<(), E>
where
    E: From<InvalidUtf8> + From<UnexpectedEndOfFile>,
    R: ReadCore<Err = E> + ?Sized,
    F: FnMut(&R) -> usize,
{
    let mut g = Guard {
        len: buf.len(),
        buf,
    };
    loop {
        if g.len == g.buf.len() {
            unsafe {
                g.buf.reserve(reservation_size(r));
                let capacity = g.buf.capacity();
                g.buf.set_len(capacity);
                r.initializer().initialize(&mut g.buf[g.len..]);
            }
        }

        match r.read(&mut g.buf[g.len..])? {
            OpRes::Eof => break,
            OpRes::Retry => {}
            OpRes::Partial(n) | OpRes::Completly(n) => g.len += n.get(),
        }
    }
    Ok(())
}

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

fn append_to_string<E, F>(buf: &mut String, f: F) -> Result<(), E>
where
    E: From<InvalidUtf8>,
    F: FnOnce(&mut Vec<u8>) -> Result<(), E>,
{
    unsafe {
        let mut g = Guard {
            len: buf.len(),
            buf: buf.as_mut_vec(),
        };
        f(g.buf)?;
        if str::from_utf8(&g.buf[g.len..]).is_err() {
            Err(E::from(InvalidUtf8))
        } else {
            g.len = g.buf.len();
            Ok(())
        }
    }
}

impl<E, T: ReadCore<Err = E>> ReadAlloc for Take<T>
where
    E: From<InvalidUtf8> + From<UnexpectedEndOfFile>,
{
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<(), E> {
        read_to_end_with_reservation(self, buf, |self_| cmp::min(self_.limit(), 32) as usize)
    }
}
