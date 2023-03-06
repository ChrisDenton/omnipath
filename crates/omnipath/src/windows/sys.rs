//! [Windows only] Use the Windows API to perform path operations.

use std::ffi::OsString;
use std::io;
use std::iter::Iterator;
use std::mem::MaybeUninit;
#[cfg(not(doc))]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr;
use std::string::String;
use std::vec::Vec;

use super::kind::{ParsedUtf8Path, Win32Absolute, Win32Relative, WinPathKind};

/// [Windows only] Extension functions that use the Windows API to resolve paths.
pub trait WinPathExt: Sealed {
    /// Makes the path absolute without resolving symlinks.
    ///
    /// Unlike canonicalize the path does not need to exist. This will also not
    /// return verbatim paths unless given one.
    ///
    /// # Example
    ///
    /// ```
    /// #[cfg(windows)]
    /// {
    ///     use omnipath::windows::WinPathExt;
    ///     use std::path::Path;
    ///     use std::env::current_dir;
    ///
    ///     let path = Path::new(r"path\to\file");
    ///     assert_eq!(
    ///         path.win_absolute().unwrap(),
    ///         // WARNING! Depending on the path, this may not always be equivalent.
    ///         current_dir().unwrap().join(path)
    ///     )
    /// }
    /// ```
    fn win_absolute(&self) -> io::Result<PathBuf>;

    /// Clean the path without making it absolute or changing its path prefix.
    ///
    /// This does the same cleaning as occurs when you pass a path to the Windows API.
    ///
    /// # Limitations
    ///
    /// This function will currently discard all `..` components after cleaning.
    ///
    /// # Example
    ///
    /// ```
    /// #[cfg(windows)]
    /// {
    ///     use omnipath::windows::WinPathExt;
    ///     use std::path::Path;
    ///
    ///     let path = Path::new(r"path\.\from\..\to\\\\file..  ..");
    ///     assert_eq!(
    ///         path.win_clean().unwrap(),
    ///         Path::new(r"path\to\file")
    ///     );
    /// }
    /// ```
    #[doc(hidden)]
    fn win_clean(&self) -> io::Result<PathBuf>;

    /// Convert a verbatim path to a win32 path.
    ///
    /// If the path is not verbatim the the path is returned as-is.
    ///
    /// # Example
    ///
    /// ```
    /// #[cfg(windows)]
    /// {
    ///     use omnipath::windows::WinPathExt;
    ///     use std::path::Path;
    ///
    ///     let path = Path::new(r"\\?\C:\path\to\file.txt");
    ///     assert_eq!(
    ///         path.to_winuser_path().unwrap(),
    ///         Path::new(r"C:\path\to\file.txt")
    ///     );
    ///
    ///     let path = Path::new(r"\\?\UNC\server\share\file.txt");
    ///     assert_eq!(
    ///         path.to_winuser_path().unwrap(),
    ///         Path::new(r"\\server\share\file.txt")
    ///     );
    /// }
    /// ```
    fn to_winuser_path(&self) -> io::Result<PathBuf>;
}
impl WinPathExt for Path {
    fn win_absolute(&self) -> io::Result<PathBuf> {
        if self.as_os_str().is_empty() {
            return Ok(PathBuf::new());
        }
        if let Some(std::path::Component::Prefix(prefix)) = self.components().next() {
            if prefix.kind().is_verbatim() {
                return Ok(self.into());
            }
        }
        let path = to_wide(self)?;
        absolute_inner(&path, |path| OsString::from_wide(path).into())
    }
    #[doc(hidden)]
    fn win_clean(&self) -> io::Result<PathBuf> {
        let path = match self.to_str() {
            Some(path) => path,
            None => return Ok(self.into()),
        };

        // 1. split prefix
        let parsed = ParsedUtf8Path::from_utf8(path);
        if parsed.kind() == WinPathKind::Verbatim {
            // Skip on verbatim paths.
            return Ok(path.into());
        }
        let (prefix, path) = parsed.parts();

        // 2. use `absolute` on the path, using `\\.\` for the prefix
        let path = String::from_iter([r"\\.\", path]);
        let path = to_wide(Path::new(&path))?;
        absolute_inner(&path, |path| {
            // 3. replace the prefix (if any)
            let mut os_path = OsString::from(prefix);
            os_path.push(&OsString::from_wide(&path[r"\\.\".len()..]));
            os_path.into()
        })
    }
    fn to_winuser_path(&self) -> io::Result<PathBuf> {
        let path = match self.to_str() {
            Some(path) => path,
            None => return Ok(self.into()),
        };
        let (prefix, subpath) = match Win32Absolute::from_verbatim_str(path) {
            Ok(result) => result,
            Err(_) => return Ok(path.into()),
        };
        let prefix = match prefix {
            Win32Absolute::Drive(_) => return Ok(subpath.into()),
            Win32Absolute::Device => r"\\.\",
            Win32Absolute::Unc => {
                if subpath.is_empty() {
                    r"\\"
                } else {
                    r"\"
                }
            }
        };
        let mut win32 = String::with_capacity(prefix.len() + subpath.len());
        win32.push_str(prefix);
        win32.push_str(subpath);

        // Test if path is unchanged by a call to absolute.
        let win32 = Path::new(&win32);
        if win32 == win32.win_absolute().unwrap_or_default() {
            Ok(win32.into())
        } else {
            Ok(path.into())
        }
    }
}

/// [Windows only] Turns a relative Windows prefix into an absolute path.
pub fn resolve_prefix(prefix: Win32Relative) -> io::Result<PathBuf> {
    match prefix {
        Win32Relative::CurrentDirectory => Path::new(r".\").win_absolute(),
        Win32Relative::Root => Path::new(r"\").win_absolute(),
        // GetFullPathName("X:")
        Win32Relative::DriveRelative(drive) => {
            let path = [drive, b':' as u16, 0];
            absolute_inner(&path, |path| OsString::from_wide(path).into())
        }
    }
}

/// Make a non-verbatim path absolute.
fn absolute_inner<F>(path: &[u16], f: F) -> io::Result<PathBuf>
where
    F: FnOnce(&[u16]) -> PathBuf,
{
    debug_assert!(!path.starts_with(&[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16]));
    assert_eq!(path.last(), Some(&0));
    unsafe {
        const MAX_PATH: u16 = i16::MAX as u16;
        let mut buffer: [MaybeUninit<u16>; MAX_PATH as usize] = MaybeUninit::uninit().assume_init();
        let capacity = MAX_PATH as u32;
        let len = c::GetFullPathNameW(
            path.as_ptr(),
            capacity,
            buffer.as_mut_ptr().cast(),
            ptr::null_mut(),
        );
        if len == 0 {
            Err(io::Error::last_os_error())
        } else {
            let path = &*((&buffer[..len as usize]) as *const _ as *const [u16]);
            Ok(f(path))
        }
    }
}

fn to_wide(path: &Path) -> io::Result<Vec<u16>> {
    let mut contains_null = false;
    let path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .inspect(|&w| {
            if w == 0 {
                contains_null = true
            }
        })
        .chain([0])
        .collect();
    if !contains_null {
        Ok(path)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "paths must not contain nulls"))
    }
}

#[allow(nonstandard_style, clippy::style)]
mod c {
    type DWORD = u32;
    type LPCWSTR = *const u16;
    type LPWSTR = *mut u16;
    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetFullPathNameW(
            lpFileName: LPCWSTR,
            nBufferLength: DWORD,
            lpBuffer: LPWSTR,
            lpFilePart: *mut LPWSTR,
        ) -> DWORD;
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for std::path::Path {}
}
use private::Sealed;
