#![cfg(any(doc, all(unix, feature = "std")))]
use std::env;
use std::io;
#[cfg(not(doc))]
use std::os::unix::ffi::OsStrExt;
use std::path::Component;
use std::path::{Path, PathBuf};

pub trait PosixPathExt: Sealed {
	/// [Unix only] Make a POSIX path absolute without changing its semantics.
	///
	/// Unlike canonicalize the path does not need to exist. Symlinks and `..`
	/// components will not be resolved.
	///
	/// # Example
	///
	/// ```
	/// #[cfg(unix)]
	/// {
	///     use omnipath::posix::PosixPathExt;
	///     use std::path::Path;
	///     use std::env::current_dir;
	///
	///     let path = Path::new(r"path/to/..//./file");
	///     assert_eq!(
	///         path.posix_absolute().unwrap(),
	///         current_dir().unwrap().join("path/to/../file")
	///     )
	/// }
	/// ```
	fn posix_absolute(&self) -> io::Result<PathBuf>;

	/// [Unix only] Make a POSIX path lexically absolute.
	///
	/// Unlike `canonicalize` the path does not need to exist. Symlinks will not be resolved.
	/// Unlike [`posix_absolute`] this resolves `..` components by popping the
	/// parent component. This means that it may resolve to a different path
	/// than would be resolved by passing the path directly to the OS.
	///
	/// Usually this is not the preferred behaviour.
	///
	/// # Example
	///
	/// ```
	/// #[cfg(unix)]
	/// {
	///     use omnipath::posix::PosixPathExt;
	///     use std::path::Path;
	///     use std::env::current_dir;
	///
	///     let path = Path::new(r"path/to/..//./file");
	///     assert_eq!(
	///         path.posix_lexically_absolute().unwrap(),
	///         current_dir().unwrap().join("path/file")
	///     )
	/// }
	/// ```
	fn posix_lexically_absolute(&self) -> io::Result<PathBuf>;
}
impl PosixPathExt for Path {
	fn posix_absolute(&self) -> io::Result<PathBuf> {
		// This is mostly a wrapper around collecting `Path::components`, with
		// exceptions made where this conflicts with the POSIX specification.
		// See 4.13 Pathname Resolution, IEEE Std 1003.1-2017
		// https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap04.html#tag_04_13

		// Get the components, skipping the redundant leading "." component if it exists.
		let mut components = self.strip_prefix(".").unwrap_or(self).components();
		let path_os = self.as_os_str().as_bytes();

		let mut normalized = if self.is_absolute() {
			// "If a pathname begins with two successive <slash> characters, the
			// first component following the leading <slash> characters may be
			// interpreted in an implementation-defined manner, although more than
			// two leading <slash> characters shall be treated as a single <slash>
			// character."
			if path_os.starts_with(b"//") && !path_os.starts_with(b"///") {
				components.next();
				PathBuf::from("//")
			} else {
				PathBuf::new()
			}
		} else {
			env::current_dir()?
		};
		normalized.extend(components);

		// "Interfaces using pathname resolution may specify additional constraints
		// when a pathname that does not name an existing directory contains at
		// least one non- <slash> character and contains one or more trailing
		// <slash> characters".
		// A trailing <slash> is also meaningful if "a symbolic link is
		// encountered during pathname resolution".
		if path_os.ends_with(b"/") {
			normalized.push("");
		}

		Ok(normalized)
	}
	fn posix_lexically_absolute(&self) -> io::Result<PathBuf> {
		// This is mostly a wrapper around collecting `Path::components`, with
		// exceptions made where this conflicts with the POSIX specification.
		// See 4.13 Pathname Resolution, IEEE Std 1003.1-2017
		// https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap04.html#tag_04_13

		// Get the components, skipping the redundant leading "." component if it exists.
		let mut components = self.strip_prefix(".").unwrap_or(self).components();
		let path_os = self.as_os_str().as_bytes();

		let mut normalized = if self.is_absolute() {
			// "If a pathname begins with two successive <slash> characters, the
			// first component following the leading <slash> characters may be
			// interpreted in an implementation-defined manner, although more than
			// two leading <slash> characters shall be treated as a single <slash>
			// character."
			if path_os.starts_with(b"//") && !path_os.starts_with(b"///") {
				components.next();
				PathBuf::from("//")
			} else {
				PathBuf::new()
			}
		} else {
			env::current_dir()?
		};
		components.for_each(|component| {
			if component == Component::ParentDir {
				normalized.pop();
			} else {
				normalized.push(component);
			}
		});

		// "Interfaces using pathname resolution may specify additional constraints
		// when a pathname that does not name an existing directory contains at
		// least one non- <slash> character and contains one or more trailing
		// <slash> characters".
		// A trailing <slash> is also meaningful if "a symbolic link is
		// encountered during pathname resolution".
		if path_os.ends_with(b"/") {
			normalized.push("");
		}

		Ok(normalized)
	}
}

mod private {
	pub trait Sealed {}
	impl Sealed for std::path::Path {}
}
use private::Sealed;
