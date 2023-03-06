// Quick macro for case-insensitive comparisons.
// Also converts
// FIXME: This should probably be replaced with uppercasing to a stack buffer.
macro_rules! pattern {
    () => {};
    (!!**!! A) => { b'a' | b'A' };
    (!!**!! C) => { b'c' | b'C' };
    (!!**!! I) => { b'i' | b'I' };
    (!!**!! L) => { b'l' | b'L' };
    (!!**!! M) => { b'm' | b'M' };
    (!!**!! N) => { b'n' | b'N' };
    (!!**!! O) => { b'o' | b'O' };
    (!!**!! P) => { b'p' | b'P' };
    (!!**!! R) => { b'r' | b'R' };
    (!!**!! T) => { b't' | b'T' };
    (!!**!! U) => { b'u' | b'U' };
    (!!**!! X) => { b'x' | b'X' };
    (!!**!! /) => { b'\\' | b'/' };
    (!!**!! '.') => { b'.' | b'?' };

    (!!**!! '$') => { b'$' };
    (!!**!! ':') => { b':' };

    (!!**!! $pat:pat) => { $pat };
    ([$($tt:tt$(@$pat:pat)?),+]) => {
        [$(pattern!(!!**!! $tt$(@$pat)?)),+]
    };
    ([$($tt:tt$(|$pat:pat)?),+]) => {
        [$(pattern!(!!**!! $tt)$(|$pat)?),+]
    };
    ($($tt:tt)|+) => {
        $(pattern!($tt))|+
    };
}

macro_rules! match_pattern {
    ($match:expr; $([$($tt:tt)+]$(|[$($tt2:tt)*])* => $expr:expr),+, _ => $final:expr) => {
        match $match {
            $(
                pattern!([$($tt)+]$(|[$($tt2)*])*) => $expr
            ),+,
            _ => $final,
        }
    };
}

/// Trim `len` bytes from the start, `const` edition.
///
/// This should only be used for small sizes of `len`.
pub const fn trim_start(bytes: &[u8], len: u8) -> &[u8] {
    if len == 0 {
        return bytes;
    }
    match bytes {
        [_, bytes @ ..] => trim_start(bytes, len - 1),
        [] => bytes,
    }
}

/// Trim `len` bytes from the start of a `&str`.
///
/// SAFETY: The trimmed str must be valid UTF-8.
pub const unsafe fn trim_start_str(s: &str, len: usize) -> &str {
    core::str::from_utf8_unchecked(trim_start(s.as_bytes(), len as u8))
}

/// Get the length of a UTF-8 encoded scalar by examining the leading byte.
pub const fn utf8_len(first_byte: u8) -> u8 {
    match first_byte.leading_ones() as u8 {
        0 => 1,
        n => n,
    }
}

/// Convert a BMP UTF-8 encoded code point to a UTF-16 encoded code point.
///
/// While it is safe to call this with random bytes or non-BMP code points,
/// the result is unspecified.
pub const fn bmp_utf8_to_utf16(bytes: &[u8]) -> u16 {
    debug_assert!(matches!(utf8_len(bytes[0]), 1 | 2 | 3));
    match utf8_len(bytes[0]) {
        3.. => {
            let a = (bytes[0] & 0b1111) as u16;
            let b = (bytes[1] & 0b111111) as u16;
            let c = (bytes[2] & 0b111111) as u16;
            (a << 12) | (b << 6) | c
        }
        2.. => {
            let a = (bytes[0] & 0b11111) as u16;
            let b = (bytes[1] & 0b111111) as u16;
            (a << 6) | b
        }
        _ => bytes[0] as u16,
    }
}
