use core::cmp;
use core::fmt;
use core::num::NonZeroUsize;
use core::ptr;
use core::slice;

#[derive(Copy, Clone, Debug)]
pub struct UnexpectedEndOfFile;

#[derive(Copy, Clone, Debug)]
pub struct InvalidUtf8;

#[derive(Copy, Clone, Debug)]
pub struct OutOfBounds;

#[derive(Copy, Clone, Debug)]
pub struct FormatterError;

/// Response of a Read or Write Operation
pub enum OpRes {
    /// Operation did not complete and should be retried.
    Retry,
    /// Render was completly read and does not have any more data.
    /// To signal an zero sized buffer, use `OpRes::Completly`
    Eof,
    /// Buffer was completly filled. There may or may not be data left.
    Completly(NonZeroUsize),
    /// Buffer was partial filled. There may or may not be data left.
    Partial(NonZeroUsize),
}

pub trait ReadCore {
    type Err: From<UnexpectedEndOfFile> + From<InvalidUtf8>;

    fn read(&mut self, buf: &mut [u8]) -> Result<OpRes, Self::Err>;

    unsafe fn initializer(&self) -> Initializer {
        Initializer::zeroing()
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Self::Err> {
        while !buf.is_empty() {
            match self.read(&mut buf)? {
                OpRes::Eof => return Err(Self::Err::from(UnexpectedEndOfFile)),
                OpRes::Retry => {}
                OpRes::Partial(n) => buf = &mut buf[n.get()..],
                OpRes::Completly(_) => break,
            }
        }
        Ok(())
    }

    #[must_use]
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Bytes { inner: self }
    }

    fn chain<R: ReadCore>(self, next: R) -> Chain<Self, R>
    where
        Self: Sized,
    {
        Chain {
            first: self,
            second: next,
            done_first: false,
        }
    }

    fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        Take { inner: self, limit }
    }
}

pub trait WriteCore {
    type Err: From<UnexpectedEndOfFile> + From<FormatterError>;

    fn write(&mut self, buf: &[u8]) -> Result<OpRes, Self::Err>;

    fn flush(&mut self) -> Result<(), Self::Err>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Self::Err> {
        while !buf.is_empty() {
            match self.write(buf)? {
                OpRes::Eof => return Err(Self::Err::from(UnexpectedEndOfFile)),
                OpRes::Retry => {}
                OpRes::Partial(n) => buf = &buf[n.get()..],
                OpRes::Completly(_) => break,
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<(), Self::Err> {
        struct Adaptor<'a, T: ?Sized + 'a, E: From<UnexpectedEndOfFile> + From<FormatterError>> {
            inner: &'a mut T,
            error: Result<(), E>,
        }

        impl<
                'a,
                T: self::WriteCore<Err = E> + ?Sized,
                E: From<UnexpectedEndOfFile> + From<FormatterError>,
            > fmt::Write for Adaptor<'a, T, E>
        {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adaptor {
            inner: self,
            error: Ok(()),
        };
        let _ = fmt::write(&mut output, fmt);
        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => {
                if output.error.is_err() {
                    output.error
                } else {
                    Err(Self::Err::from(FormatterError))
                }
            }
        }
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Debug)]
pub struct Bytes<R> {
    inner: R,
}

impl<R> Bytes<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<E, R: ReadCore<Err = E>> Iterator for Bytes<R>
where
    E: From<InvalidUtf8> + From<UnexpectedEndOfFile>,
{
    type Item = Result<u8, E>;

    fn next(&mut self) -> Option<Result<u8, E>> {
        let mut byte = 0;
        loop {
            match self.inner.read(slice::from_mut(&mut byte)) {
                Ok(OpRes::Eof) => return None,
                Ok(OpRes::Retry) => {}
                Ok(OpRes::Partial(_)) | Ok(OpRes::Completly(_)) => return Some(Ok(byte)),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

pub struct Chain<T, U> {
    first: T,
    second: U,
    done_first: bool,
}

impl<T, U> Chain<T, U> {
    pub fn new(first: T, second: U) -> Self {
        Self {
            first,
            second,
            done_first: false,
        }
    }
}

impl<T, U> Chain<T, U> {
    pub fn into_inner(self) -> (T, U) {
        (self.first, self.second)
    }

    pub fn get_ref(&self) -> (&T, &U) {
        (&self.first, &self.second)
    }

    pub fn get_mut(&mut self) -> (&mut T, &mut U) {
        (&mut self.first, &mut self.second)
    }
}

impl<T: fmt::Debug, U: fmt::Debug> fmt::Debug for Chain<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chain")
            .field("t", &self.first)
            .field("u", &self.second)
            .finish()
    }
}

impl<T: ReadCore<Err = E>, U: ReadCore<Err = E>, E> ReadCore for Chain<T, U>
where
    E: From<InvalidUtf8> + From<UnexpectedEndOfFile>,
{
    type Err = E;

    fn read(&mut self, buf: &mut [u8]) -> Result<OpRes, E> {
        if !self.done_first {
            match self.first.read(buf)? {
                OpRes::Eof => self.done_first = true,
                op => return Ok(op),
            }
        }
        self.second.read(buf)
    }

    // fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
    //     if !self.done_first {
    //         match self.first.read_vectored(bufs)? {
    //             0 if bufs.iter().any(|b| !b.is_empty()) => self.done_first = true,
    //             n => return Ok(n),
    //         }
    //     }
    //     self.second.read_vectored(bufs)
    // }

    unsafe fn initializer(&self) -> Initializer {
        let initializer = self.first.initializer();
        if initializer.should_initialize() {
            initializer
        } else {
            self.second.initializer()
        }
    }
}

pub struct Take<T> {
    inner: T,
    limit: u64,
}

impl<T> Take<T> {
    pub fn new(inner: T, limit: u64) -> Self {
        Self { inner, limit }
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<E, T: ReadCore<Err = E>> ReadCore for Take<T>
where
    E: From<InvalidUtf8> + From<UnexpectedEndOfFile>,
{
    type Err = E;

    fn read(&mut self, buf: &mut [u8]) -> Result<OpRes, Self::Err> {
        if self.limit == 0 {
            return Ok(OpRes::Eof);
        }

        let max = cmp::min(buf.len() as u64, self.limit) as usize;
        match self.inner.read(&mut buf[..max])? {
            OpRes::Partial(n) => {
                self.limit -= n.get() as u64;
                Ok(OpRes::Partial(n))
            }
            res @ OpRes::Completly(_) => {
                self.limit -= max as u64;
                Ok(res)
            }
            op => Ok(op),
        }
    }

    unsafe fn initializer(&self) -> Initializer {
        self.inner.initializer()
    }
}

#[derive(Debug)]
pub struct Initializer(bool);

impl Initializer {
    #[inline]
    pub fn zeroing() -> Initializer {
        Initializer(true)
    }

    #[inline]
    pub unsafe fn nop() -> Initializer {
        Initializer(false)
    }

    #[inline]
    pub fn should_initialize(&self) -> bool {
        self.0
    }

    #[inline]
    pub fn initialize(&self, buf: &mut [u8]) {
        if self.should_initialize() {
            unsafe { ptr::write_bytes(buf.as_mut_ptr(), 0, buf.len()) }
        }
    }
}
