#![no_std]
#![allow(clippy::single_char_pattern)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;
// Utility functions and macros.
#[macro_use]
mod util;
pub mod posix;
pub mod windows;

#[cfg(any(doc, all(unix, feature = "std")))]
pub use posix::PosixPathExt;

#[cfg(any(doc, all(windows, feature = "std")))]
#[doc(no_inline)]
pub use windows::WinPathExt;
