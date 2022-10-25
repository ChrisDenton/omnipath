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

//use alloc::{
//	borrow::{Cow, ToOwned},
//	string::String,
//};
//use core::{borrow::Borrow, fmt, ops::Deref};

// Utility functions and macros.
#[macro_use]
mod util;

//pub mod pure;
//mod raw;

pub mod posix;
pub mod pure;
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
/// use std::env::current_dir;
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

//use pure::DEFAULT_SEPARATOR;
//use windows::kind::{ParsedUtf8Path, Win32Absolute, Win32Relative, WinPathKind};
//pub type PurePathBuf = pure::PurePathBuf<DEFAULT_SEPARATOR>;
//pub type PurePath = pure::PurePath<DEFAULT_SEPARATOR>;

/*
/// A cross-platform Windows path (UTF-8 encoded).
#[derive(Eq, Ord, PartialEq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct WinUtf8Path {
	inner: str,
}
impl WinUtf8Path {
	pub const fn from_utf8(utf8: &str) -> &Self {
		// SAFETY: Self is `repr(transparent)` so we can transmute safely.
		unsafe { &*(utf8 as *const str as *const Self) }
	}

	pub fn as_utf8(&self) -> &str {
		&self.inner
	}

	/// Ensures that the path is not verbatim by converting verbatim paths
	/// to more familiar user paths.
	pub fn to_user_path(&self) -> Cow<Self> {
		let path = &self.inner;
		let (prefix, subpath) = match Win32Absolute::from_verbatim_str(path) {
			Ok(result) => result,
			Err(_) => return Cow::Borrowed(self),
		};
		// test if the path can be passed through win32 without changing.
		let subpath = raw::StrPath::from_str(subpath);
		if !subpath.is_win32_safe() {
			return Cow::Borrowed(self);
		}
		/*let test_path = subpath.trim_matches('\\');
		if !path::str_split_verbatim(test_path).all(path::str_is_win32_safe) {
			return Cow::Borrowed(self);
		}*/

		// Get the str prefix, if any.
		let prefix = match prefix {
			Win32Absolute::Drive(_) => return Cow::Borrowed(Self::from_utf8(subpath.as_str())),
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
		win32.push_str(subpath.as_str());
		Cow::Owned(WinUtf8PathBuf { inner: win32 })
	}

	/// Clean the path without making it absolute or changing its prefix.
	///
	/// This is similar to what happens automatically when you pass a path to
	/// win32 filesystem APIs.
	pub fn clean(&self) -> Cow<Self> {
		// FIXME: This function is a bit of a mess right now.
		// Ideally it should use a proper parser and also
		// return a borrowed path if the path is already clean
		// or the cleaned path is a single subslice.

		let parsed = ParsedUtf8Path::from_utf8(&self.inner);
		if parsed.kind() == WinPathKind::Verbatim {
			// if the path is verbatim, do nothing.
			return Cow::Borrowed(self);
		}
		let (prefix, path) = parsed.parts();
		let path = raw::StrPath::from_str(path);

		// This pushes to buffer in reverse as a simply way t6o handle
		// `.` and `..` components.
		let mut clean = util::StringBuilder::with_capacity(self.as_utf8().len());
		// true if there is at least one component in the path.
		let mut has_path = false;
		let mut skip = 0;
		{
			let mut iter = path.win32_ancestors(&mut skip); //path::str_rsplit_win32(path, &mut skip);
			if let Some(component) = iter.next() {
				has_path = true;
				clean.reverse_push(component.as_str())
			}
			for component in iter {
				clean.reverse_push(r"\");
				clean.reverse_push(component.as_str());
			}
		}
		// Collect together the remaining `..` components.
		// This is not needed for absolute paths or root relative paths.
		if skip > 0 && !parsed.kind().is_absolute() && parsed.kind() != WinPathKind::RootRelative {
			if clean.is_empty() {
				clean.reverse_push("..");
				skip -= 1;
			}
			for _ in 0..skip {
				clean.reverse_push(r"..\");
			}
		}
		// Try to handle broken UNC paths.
		// UNC paths should be in the form `\\server\share`
		// Broken UNC paths include paths such as `\\server`, `\\\` and `\\\share`.
		if parsed.kind() == WinPathKind::Unc {
			if has_path {
				clean.reverse_push(r"\");
			}
			if let Some((server, share)) = prefix[2..].split_once(['\\', '/']) {
				if !share.is_empty() {
					clean.reverse_push(share);
					clean.reverse_push(r"\");
				}
				clean.reverse_push(server);
			} else if prefix.len() > 2 {
				clean.reverse_push(&prefix[2..]);
			}
		}
		clean.reverse_push(parsed.normalized_str_kind().as_str());
		let mut clean = clean.finalize();

		// Clean `.` and ` ` from the end of the path.
		// The does not clean `..` paths (which may only occur for relative paths).
		clean.truncate(raw::trim_full_path(&clean).len());

		Cow::Owned(WinUtf8PathBuf { inner: clean })
	}

	/// Returns a (prefix, subpath) pair.
	pub fn split_prefix(&self) -> (&Self, &Self) {
		let (prefix, path) = ParsedUtf8Path::from_utf8(self.as_utf8()).parts();
		(Self::from_utf8(prefix), Self::from_utf8(path))
	}
}
impl fmt::Display for WinUtf8Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_utf8())
	}
}
impl fmt::Debug for WinUtf8Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_utf8().fmt(f)
	}
}

impl ToOwned for WinUtf8Path {
	type Owned = WinUtf8PathBuf;
	fn to_owned(&self) -> Self::Owned {
		WinUtf8PathBuf { inner: self.as_utf8().into() }
	}
}
impl Borrow<WinUtf8Path> for WinUtf8PathBuf {
	fn borrow(&self) -> &WinUtf8Path {
		WinUtf8Path::from_utf8(&self.inner)
	}
}

/// A buffer for [`WinUtf8Path`].
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Hash, Default)]
pub struct WinUtf8PathBuf {
	inner: String,
}
impl WinUtf8PathBuf {
	pub const fn new() -> Self {
		WinUtf8PathBuf { inner: String::new() }
	}
}
impl Deref for WinUtf8PathBuf {
	type Target = WinUtf8Path;
	fn deref(&self) -> &WinUtf8Path {
		WinUtf8Path::from_utf8(&self.inner)
	}
}

/// A collection of components that make up a Windows path without any prefix.
#[derive(Debug)]
#[repr(transparent)]
pub struct WinSubPath {
	path: raw::StrPath,
}
impl WinSubPath {
	pub const fn is_empty(&self) -> bool {
		self.path.is_empty()
	}

	const fn from_win_str_path(path: &raw::StrPath) -> &Self {
		unsafe { &*(path as *const _ as *const Self) }
	}
	pub fn parent(&self) -> Option<&Self> {
		self.path.parent_verbatim().map(WinSubPath::from_win_str_path)
	}
	pub fn extension(&self) -> Option<&str> {
		self.filename().and_then(|c| c.extension())
	}
	pub fn extensions(&self) -> Option<impl Iterator<Item = &str>> {
		self.filename().and_then(|c| c.extensions())
	}
	pub fn iter(&self) -> impl Iterator<Item = Component<'_>> {
		self.path.verbatim_components().map(|c| Component::new_unchecked(c.as_str()))
	}
	pub fn ancestors(&self) -> impl Iterator<Item = Component<'_>> {
		self.path.reverse_verbatim_components().map(|c| Component::new_unchecked(c.as_str()))
	}
	pub fn filename(&self) -> Option<Component<'_>> {
		self.ancestors().next()
	}
	pub fn display(&self) -> DisplayPath<'_> {
		DisplayPath { force_unix_separators: false, path: &self.path }
	}
}

/// A mutable buffer for `WinSubPath`.
#[derive(Debug)]
pub struct WinSubPathBuf {
	path: raw::StrPathBuffer,
}
impl WinSubPathBuf {
	/// Push the component, cleaning the path.
	///
	/// Trailing `.` or ` ` will be trimmed. This mimics the way the win32 APIs
	/// will trim paths passed to filesystem APIs.
	///
	/// A single `.` is interpreted to mean the current directory and will
	/// simply strip any trailing path separator. `..` components will be
	/// interpreted as the parent directory and is equivalent to calling `pop_current`.
	///
	/// Returns true if the component was trimmed before appending.
	pub fn push(&mut self, component: Component) -> bool {
		match component.name {
			"." => {
				self.remove_trailing_separator();
				false
			}
			".." => {
				self.pop_current();
				false
			}
			_ => {
				let trimmed = component.as_win_str_path().trim_filename();
				self.path.push_component(trimmed);
				trimmed.len() != component.name.len()
			}
		}
	}

	/// Push the component without doing any cleaning.
	///
	/// Any `.` and `..` components will be taken as literal file or directory
	/// names. If the path is not representable as a user path then it should
	/// be displayed as verbatim.
	pub fn push_verbatim(&mut self, component: Component) {
		self.path.push_component(component.as_win_str_path());
	}

	pub fn extend<'i, I: Iterator<Item = Component<'i>>>(&mut self, components: I) {
		components.for_each(|c| {
			self.push(c);
		})
	}

	pub fn extend_verbatim<'i, I: Iterator<Item = Component<'i>>>(&mut self, components: I) {
		components.for_each(|c| {
			self.push_verbatim(c);
		})
	}

	/// Remove the last component. If the path ends with a path separator,
	/// this will only remove the trailing separator. Otherwise this is the
	/// same as `pop_to_parent`.
	pub fn pop(&mut self) -> bool {
		self.path.pop_verbatim_component()
	}

	/// Truncates to the parent directory.
	pub fn pop_current(&mut self) -> bool {
		self.remove_trailing_separator() || self.pop()
	}

	/// Strips any trailing separator.
	pub fn remove_trailing_separator(&mut self) -> bool {
		if self.path.ends_with_char('\\') {
			self.pop();
			true
		} else {
			false
		}
	}
}
impl Deref for WinSubPathBuf {
	type Target = WinSubPath;
	fn deref(&self) -> &Self::Target {
		WinSubPath::from_win_str_path(&self.path)
	}
}

/// Asserts that the component doesn't contain any path separators.
#[derive(Clone, Copy)]
pub struct Component<'a> {
	name: &'a str,
}
impl<'a> Component<'a> {
	pub fn as_str(&self) -> &str {
		self.name
	}
	pub fn is_empty(self) -> bool {
		self.name.is_empty()
	}
	pub fn extension(self) -> Option<&'a str> {
		self.extensions().and_then(|mut ext| ext.next())
	}
	pub fn extensions(self) -> Option<impl Iterator<Item = &'a str>> {
		let name = self.name.strip_prefix('.').unwrap_or(self.name);
		if let Some((_, extensions)) = name.split_once('.') {
			Some(extensions.rsplit('.'))
		} else {
			None
		}
	}
	pub fn new(name: &'a str) -> Option<Self> {
		if name.contains(['\\', '/']) {
			None
		} else {
			Some(Self { name })
		}
	}
	pub fn new_unix(name: &'a str) -> Option<Self> {
		if name.contains('/') {
			None
		} else {
			Some(Self { name })
		}
	}
	pub fn new_verbatim(name: &'a str) -> Option<Self> {
		if name.contains('\\') {
			None
		} else {
			Some(Self { name })
		}
	}
	pub fn new_unchecked(name: &'a str) -> Self {
		Self { name }
	}
	fn as_win_str_path(self) -> &'a raw::StrPath {
		raw::StrPath::from_str(self.name)
	}
}

pub struct DisplayPath<'a> {
	force_unix_separators: bool,
	path: &'a raw::StrPath,
}
impl fmt::Display for DisplayPath<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.path.win_display(f, self.force_unix_separators)
	}
}

// IDEAS:
// * PureComponent<const SEPARATOR: char>
// * PurePath<const SEPARATOR: char>
// Modify in place (use closure, return Some(val) to modify)
*/
