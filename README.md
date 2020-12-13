- Feature Name: `read_and_write_core`
- Start Date: 2020-12-13
- RFC PR: [rust-lang/rfcs#0000](https://github.com/rust-lang/rfcs/pull/0000)
- Rust Issue: [rust-lang/rust#0000](https://github.com/rust-lang/rust/issues/0000)

# Summary
[summary]: #summary

This RFC aims to provide `Read` and `Write` Traits for the `core` and `alloc` module while trying to keep backward compatibility with the current `Read` Trait.

This RFC requires:
* [Specialization RFC](https://github.com/rust-lang/rfcs/pull/1210)
* [Specialization Issue](https://github.com/rust-lang/rust/issues/31844)

# Motivation
[motivation]: #motivation

Currently, there are no official `Read` and `Write` traits available for `no_std` environments. These traits, however, are crucial for providing general layouts developers agree on when transferring data via a network, storage, or other sources.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

There are two new traits called something like `ReadCore` and `ReadExt` which the user can implement. 

`ReadExt` resides in alloc and provides methods that require `Vec` or `String`. Its functions are automatically implemented for all `ReadCore` Types. 

The `ReadCore` Type provides methods that do not rely on any allocation and thus provide a structured way to read into a Stack allocated array. 

Also, the `Read` trait implements `ReadCore` to allow using Read in methods that require a `ReadCore` implementation.

The implementation for `Write` is similar to the one for `Read`.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

There will be five changes in `core`, `alloc` and `std`:

Core gets a new trait called something like `ReadCore`:

```rust
pub trait ReadCore {
    type Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        // Implementation from std::io::Read
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        // Implementation from std::io::Read
    }
}
```

Alloc gets a new trait called something like `ReadExt`. In addition it gets a blanket implementation for every type implementing `Read`:

```rust
use alloc::string::String;
use alloc::vec::Vec;
use core::io::ReadCore;

pub trait ReadExt: ReadCore {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error>;

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Self::Error>;
}

impl<T: ?Sized> ReadExt for T where T: ReadCore {
    default fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }

    default fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Self::Error> {
        // Implementation from std::io::Read
        unimplemented!();
    }
}
```

And in `std` the `Read` trait is adapted to require `ReadCore`. `ReadCore` and `ReadExt` get a blanked implementation for every `Read`:

```rust
use alloc::io::ReadExt;
use core::io::ReadCore;
use std::io::Error;

pub trait Read: ReadCore<Error = Error> {
    [...]
}

impl<T: ?Sized> ReadCore for T where T: Read {
    type Error = Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Read::read(self, buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        Read::read_exact(self, buf)
    }
}

impl<T: ?Sized> ReadExt for T where T: Read {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Self::Error> {
        Read::read_to_end(self, buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Self::Error> {
        Read::read_to_string(self, buf)
    }
}
```

# Drawbacks
[drawbacks]: #drawbacks

While it is possible to use a `Read` in a function requiring `ReadCore` or `ReadExt` (due to blanket implementation), it is not possible to use a `ReadCore` for a function requiring a `Read`. This, however, can be solved by using a Compatibility Layer like the following:

```rust
use alloc::io::ReadExt;
use core::io::ReadCore;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind, Read};

#[derive(Debug)]
struct LegacyError<E: fmt::Debug>(E);

impl<E: fmt::Debug> StdError for LegacyError<E> {}

impl<E: fmt::Debug> fmt::Display for LegacyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to read from Reader. Error: {:?}", self.0)
    }
}

struct LegacyRead<R: ReadExt>(R);

impl<Error, Reader> Read for LegacyRead<Reader>
where
    Error: fmt::Debug + Send + Sync + 'static,
    Reader: ReadExt + ReadCore<Error = Error>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.0
            .read(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, IoError> {
        self.0
            .read_to_end(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, IoError> {
        self.0
            .read_to_string(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), IoError> {
        self.0
            .read_exact(buf)
            .map_err(|err| IoError::new(ErrorKind::Other, LegacyError(err)))
    }
}
```

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Alternatives are using or writing crates like [not-io](https://crates.io/crates/not-io), which, however, do not sufficiently provide commonly agreed traits and thus may require Transformation types. 

The impact of not providing common types for core and alloc is, that e.g. microcontroller vendors will create custom traits making interoperability difficult and painful. 

# Future possibilities
[future-possibilities]: #future-possibilities

Long term, this RFC paves the road to deprecating `Read` and `Write` and fully porting them to `core` and/or `alloc`, bringing the full power of rust to the system developing world.

It also may lead the way to breaking off more functionality from `std` and putting them into dedicated sub-crates like `net` and `io`, which would allow a target to exactly specify what is supported and what not (one example is web assembly which works with `std`, but uses `unimplemented` in a lot of methods).
