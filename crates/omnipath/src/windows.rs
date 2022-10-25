pub(crate) mod kind;
#[cfg(any(doc, all(windows, feature = "std")))]
mod sys;

use alloc::string::String;

#[cfg(any(doc, all(windows, feature = "std")))]
pub use sys::{resolve_prefix, WinPathExt};

pub use kind::{Win32Relative, WinPathKind};

use crate::pure::{Component, PurePathBuf};

const WINDOWS_SEPARATOR: char = '\\';

/// The different kinds of prefixes.
#[derive(Clone, Copy, Debug)]
pub enum Win32Prefix {
	/// A traditional drive path such as `C:\`, `R:\`, etc.
	Drive(u16),
	/// A path to a network directory such as `\\server\share\`.
	Unc,
	/// A device path such as `\\.\COM1`.
	Device,
	/// A path that's relative to the current directory.
	CurrentDir,
}

///
pub struct WindowsPath {
	// Implementation note: This will eventually be a simpler wrapper around a String or a WideString,
	prefix: String,
	path: PurePathBuf<WINDOWS_SEPARATOR>,
}
impl WindowsPath {
	pub fn new() -> Self {
		Self { prefix: String::new(), path: PurePathBuf::new() }
	}

	pub fn print(&self) {
		std::print!("{}", &self.prefix);
		std::println!("{}", self.path.display());
	}

	/// Parse a Windows path from a str.
	pub fn parse(path: &str) -> Self {
		// Parse the path kind.
		let parsed = kind::ParsedUtf8Path::from_utf8(path);
		let (prefix, mut path) = parsed.parts();
		// trim the end of the path
		if let Some(fname) = path.rsplit(['/', '\\']).next() {
			if fname != "." && fname != ".." {
				path = path.trim_end_matches(['.', ' ']);
			}
		}
		let mut this = Self { prefix: prefix.into(), path: PurePathBuf::new() };
		// normalize the prefix's separators.
		for b in unsafe { this.prefix.as_bytes_mut() } {
			if *b == b'/' {
				*b = b'\\';
			}
		}
		if parsed.kind().is_absolute() && !this.prefix.ends_with('\\') {
			this.prefix.push('\\');
		}
		for component in path.split(['/', '\\']) {
			let component = WindowsComponent::new_unchecked(component).clean_dir_name();
			this.push(component);
		}
		this
	}

	pub fn push(&mut self, component: WindowsComponent) {
		match component.as_str() {
			"." => {
				self.pop_empty();
			}
			".." => {
				self.pop_empty();
				self.pop();
			}
			name => {
				self.path.push(Component::new_unchecked(name));
			}
		}
	}
	pub fn pop_empty(&mut self) -> bool {
		if self.path.is_file_name_empty() {
			self.pop()
		} else {
			false
		}
	}
	pub fn pop(&mut self) -> bool {
		self.path.pop()
	}

	pub fn kind(&self) -> WinPathKind {
		WinPathKind::from_str(&self.prefix)
	}
}
impl Default for WindowsPath {
	fn default() -> Self {
		Self::new()
	}
}

// WindowsPath
// display, parent_dir, current_dir

#[derive(Copy, Clone, Debug)]
pub struct WindowsComponent<'a> {
	component: Component<'a, '\\'>,
}
impl<'a> WindowsComponent<'a> {
	fn new_unchecked(name: &'a str) -> Self {
		Self { component: Component::new_unchecked(name) }
	}
	pub fn clean_dir_name(self) -> Self {
		let s = self.component.as_str();
		if s != "." && s.ends_with('.') && !s.ends_with("..") {
			WindowsComponent { component: Component::new_unchecked(&s[..s.len() - 1]) }
		} else {
			self
		}
	}
	pub fn clean_file_name(self) -> Self {
		let s = self.component.as_str();
		if s != "." && s != ".." {
			WindowsComponent { component: Component::new_unchecked(s.trim_end_matches([' ', '.'])) }
		} else {
			self
		}
	}
	fn as_str(&self) -> &str {
		self.component.as_str()
	}
}
