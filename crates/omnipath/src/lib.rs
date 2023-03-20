//!
//! # Absolute paths
//!
//! [`sys_absolute`] allows you to make a path absolute without needing the path to exist.
//! This uses the rules of the current platform.
//!
//! ```
//! // Makes a path absolute acording to the rules of the current OS.
//! // It does not resolve symlinks and on Windows it won't return a verbatim
//! // path unless given one.
//! omnipath::sys_absolute("path/to/.//file".as_ref());
//! ```
//!
//! # Windows verbatim paths
//!
//! Verbatim paths are paths that start with `\\?\`. For example `\\?\C:\path\to\file`.
//! These paths can be used in Windows API calls but they are not what the user expects to see.
//! You can turn these paths into more familiar user paths using the [`to_winuser_path`](crate::WinPathExt::to_winuser_path).
//!
//! ```
//! #[cfg(windows)]
//! {
//!     use omnipath::windows::WinPathExt;
//!     use std::path::Path;
//!
//!     let verbatim = Path::new(r"\\?\C:\path\to\file.txt");
//!     let user_path = verbatim.to_winuser_path();
//!     // user_path is Ok(r"C:\path\to\file.txt")
//! }
//! ```
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

/// Converts a path to absolute according to the rules of the current platform.
///
/// Unlike [`std::fs::canonicalize`] this does not resolve symlinks.
///
/// # Example
///
/// ```
/// use omnipath::sys_absolute;
/// use std::path::Path;
/// use std::env::current_dir;
/// let path = Path::new(r"path/to/.//file");
/// assert_eq!(
///     sys_absolute(path).unwrap(),
///     // WARNING: This may not always be equal depending on the current
///     // directory and OS.
///     current_dir().unwrap().join("path/to/file")
/// );
/// ```
#[cfg(feature = "std")]
pub fn sys_absolute(path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    #[cfg(unix)]
    return PosixPathExt::posix_absolute(path);
    #[cfg(windows)]
    return WinPathExt::win_absolute(path);
}

/// Canonicalizes a path.
///
/// This is the same as [`std::fs::canonicalize`] but on Windows this attempts
/// to not return verbatim paths.
///
/// # Example
///
/// ```no_run
/// use omnipath::sys_canonicalize;
/// use std::path::Path;
///
/// # fn main() -> std::io::Result<()> {
/// let path = Path::new(r"path/to/file");
/// let canonical = sys_canonicalize(path)?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "std")]
pub fn sys_canonicalize(path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    #[cfg(unix)]
    return path.canonicalize();
    #[cfg(windows)]
    return path.canonicalize()?.to_winuser_path();
}
