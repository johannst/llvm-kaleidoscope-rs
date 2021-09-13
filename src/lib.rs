use std::convert::TryFrom;

pub mod codegen;
pub mod lexer;
pub mod llvm;
pub mod parser;

/// Fixed size of [`SmallCStr`] including the trailing `\0` byte.
pub const SMALL_STR_SIZE: usize = 16;

/// Small C string on the stack with fixed size [`SMALL_STR_SIZE`].
#[derive(Debug, PartialEq)]
pub struct SmallCStr([u8; SMALL_STR_SIZE]);

impl SmallCStr {
    /// Create a new C string from `src`.
    /// Returns [`None`] if `src` exceeds the fixed size or contains any `\0` bytes.
    pub fn new<T: AsRef<[u8]>>(src: &T) -> Option<SmallCStr> {
        let src = src.as_ref();
        let len = src.len();

        // Check for \0 bytes.
        let contains_null = unsafe { !libc::memchr(src.as_ptr().cast(), 0, len).is_null() };

        if contains_null || len > SMALL_STR_SIZE - 1 {
            None
        } else {
            let mut dest = [0; SMALL_STR_SIZE];
            dest[..len].copy_from_slice(src);
            Some(SmallCStr(dest))
        }
    }

    /// Return pointer to C string.
    pub const fn as_ptr(&self) -> *const libc::c_char {
        self.0.as_ptr().cast()
    }
}

impl TryFrom<&str> for SmallCStr {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        SmallCStr::new(&value).ok_or(())
    }
}

/// Either type, for APIs accepting two types.
pub enum Either<A, B> {
    A(A),
    B(B),
}

#[cfg(test)]
mod test {
    use super::{SmallCStr, SMALL_STR_SIZE};
    use std::convert::TryInto;

    #[test]
    fn test_create() {
        let src = "\x30\x31\x32\x33";
        let scs = SmallCStr::new(&src).unwrap();
        assert_eq!(&scs.0[..5], &[0x30, 0x31, 0x32, 0x33, 0x00]);

        let src = b"abcd1234";
        let scs = SmallCStr::new(&src).unwrap();
        assert_eq!(
            &scs.0[..9],
            &[0x61, 0x62, 0x63, 0x64, 0x31, 0x32, 0x33, 0x34, 0x00]
        );
    }

    #[test]
    fn test_contain_null() {
        let src = "\x30\x00\x32\x33";
        let scs = SmallCStr::new(&src);
        assert_eq!(scs, None);

        let src = "\x30\x31\x32\x33\x00";
        let scs = SmallCStr::new(&src);
        assert_eq!(scs, None);
    }

    #[test]
    fn test_too_large() {
        let src = (0..SMALL_STR_SIZE).map(|_| 'a').collect::<String>();
        let scs = SmallCStr::new(&src);
        assert_eq!(scs, None);

        let src = (0..SMALL_STR_SIZE + 10).map(|_| 'a').collect::<String>();
        let scs = SmallCStr::new(&src);
        assert_eq!(scs, None);
    }

    #[test]
    fn test_try_into() {
        let src = "\x30\x31\x32\x33";
        let scs: Result<SmallCStr, ()> = src.try_into();
        assert!(scs.is_ok());

        let src = (0..SMALL_STR_SIZE).map(|_| 'a').collect::<String>();
        let scs: Result<SmallCStr, ()> = src.as_str().try_into();
        assert!(scs.is_err());

        let src = (0..SMALL_STR_SIZE + 10).map(|_| 'a').collect::<String>();
        let scs: Result<SmallCStr, ()> = src.as_str().try_into();
        assert!(scs.is_err());
    }
}
