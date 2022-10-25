use alloc::string::String;
use core::fmt::{self, Write};
use core::ops::{Deref, Not};

/// The default path separator.
///
/// With the `std` feature enabled, this is the same
/// as [`std::path::MAIN_SEPARATOR`]. Without it's currently `\` on Windows and
/// `/` on all other systems. THis may change in the future as platform support
/// is improved.
#[cfg(feature = "std")]
pub const DEFAULT_SEPARATOR: char = std::path::MAIN_SEPARATOR;
#[cfg(not(feature = "std"))]
pub const DEFAULT_SEPARATOR: char = if cfg!(windows) { '\\' } else { '/' };

/// An owned [`PurePath`]/
#[derive(Default)]
pub struct PurePathBuf<const SEPARATOR: char = DEFAULT_SEPARATOR> {
	path: String,
}
impl<const SEPARATOR: char> PurePathBuf<SEPARATOR> {
	/// Create an empty buffer.
	pub fn new() -> Self {
		Self { path: String::new() }
	}

	/// Appends a single component to the path.
	pub fn push(&mut self, component: Component<SEPARATOR>) -> &mut Self {
		// Push a separator if the path is not empty and the last component
		// is not empty (i.e. the path already ends with a separator).
		if !(self.is_empty() || self.path.ends_with(SEPARATOR)) {
			self.path.push(SEPARATOR);
		}
		self.path.push_str(component.as_str());
		self
	}

	/// Remove the last component from the path.
	///
	/// If the path is already empty then this will return false.
	/// If the path ends with `SEPARATOR` then this will remove the separator.
	pub fn pop(&mut self) -> bool {
		if let Some(parent) = self.parent() {
			self.path.truncate(parent.path.len());
			true
		} else {
			false
		}
	}

	/// Empty the buffer.
	pub fn clear(&mut self) {
		self.path.clear();
	}
}
impl<const SEPARATOR: char> Deref for PurePathBuf<SEPARATOR> {
	type Target = PurePath<SEPARATOR>;
	fn deref(&self) -> &Self::Target {
		PurePath::from_str_unchecked(&self.path)
	}
}

/// A collection of [`Component`]s.
#[repr(transparent)]
#[derive(Debug)]
pub struct PurePath<const SEPARATOR: char = DEFAULT_SEPARATOR> {
	path: str,
}
impl<const SEPARATOR: char> PurePath<SEPARATOR> {
	/// Create a new empty path.
	pub fn new<'a>() -> &'a Self {
		Self::from_str_unchecked("")
	}

	fn from_str_unchecked(path: &str) -> &Self {
		unsafe { &*(path as *const str as *const Self) }
	}

	/// Whether the path is empty.
	pub fn is_empty(&self) -> bool {
		self.path.is_empty()
	}

	/// Whether the path ends with a separator.
	pub fn is_file_name_empty(&self) -> bool {
		self.is_empty() || self.path.ends_with(SEPARATOR)
	}

	/// Get the file name including extension.
	///
	/// This may return an empty component if the path ends with `SEPARATOR`.
	pub fn last(&self) -> Option<Component<SEPARATOR>> {
		self.ancestors().next().map(iter::Component::component)
	}

	/// Try to get the parent path.
	pub fn parent(&self) -> Option<&PurePath<SEPARATOR>> {
		self.ancestors().next().map(iter::Component::parent)
	}

	/// Iterate over the components of a path.
	pub fn components(&self) -> Components<SEPARATOR> {
		Components::new(&self.path)
	}

	/// Iterate over the path and its parent paths.
	///
	/// This is equivalent to calling `parent` in a loop.
	pub fn ancestors(&self) -> Ancestors<SEPARATOR> {
		Ancestors::new(&self.path)
	}

	/// Returns a display object that can be printed.
	pub fn display(&self) -> DisplayPath<SEPARATOR> {
		DisplayPath::new(self)
	}
}
impl<const S1: char, const S2: char> PartialEq<PurePath<S1>> for PurePath<S2> {
	// It would be good to specialize the S1 == S2 case.
	fn eq(&self, other: &PurePath<S1>) -> bool {
		self.ancestors()
			.map(iter::Component::component)
			.eq(other.ancestors().map(iter::Component::component))
	}
}

#[derive(Debug, Clone)]
pub struct DisplayPath<'a, const SEPARATOR: char> {
	path: &'a PurePath<SEPARATOR>,
	separator: char,
}
impl<'a, const SEPARATOR: char> DisplayPath<'a, SEPARATOR> {
	fn new(path: &'a PurePath<SEPARATOR>) -> Self {
		Self { path, separator: SEPARATOR }
	}

	/// Display the path with the given `separator`.
	///
	/// This will fail if the path already contains the `separator`.
	pub fn separator(mut self, separator: char) -> Result<Self, Self> {
		if separator != SEPARATOR && !self.path.path.contains(separator) {
			self.separator = separator;
			Ok(self)
		} else {
			Err(self)
		}
	}
}
impl<'a, const SEPARATOR: char> fmt::Display for DisplayPath<'a, SEPARATOR> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.separator != SEPARATOR {
			let mut components = self.path.components();
			if let Some(component) = components.next() {
				f.write_str(component.as_str())?;
			}
			for component in components {
				f.write_char(self.separator)?;
				f.write_str(component.as_str())?;
			}
			Ok(())
		} else {
			self.path.path.fmt(f)
		}
	}
}

/// A single component of a path.
///
/// A path is a collection of components.
///
/// The only restriction here
#[derive(Debug, Clone, Copy, Eq)]
pub struct Component<'a, const SEPARATOR: char> {
	name: &'a str,
}
impl<'a, 'b, const S1: char, const S2: char> PartialEq<Component<'a, S1>> for Component<'b, S2> {
	fn eq(&self, other: &Component<'a, S1>) -> bool {
		self.name == other.name
	}
}
impl<'a, const SEPARATOR: char> Component<'a, SEPARATOR> {
	/// Returns `None` if the `name` contains `SEPARATOR`.
	pub fn new(name: &'a str) -> Option<Self> {
		name.contains(SEPARATOR).not().then_some(Self { name })
	}

	/// Creates a component without checking.
	///
	/// Callers *must* ensure the `name` does not contain `SEPARATOR`.
	pub fn new_unchecked(name: &'a str) -> Self {
		Self { name }
	}

	/// Return the component as a `str`.
	pub fn as_str(self) -> &'a str {
		self.name
	}

	/// If the component is the empty string.
	///
	/// Only the final component of a path may be empty.
	pub fn is_empty(self) -> bool {
		self.name.is_empty()
	}

	/// Return the file_name without the final extension (if any).
	pub fn file_name(self) -> &'a str {
		self.extensions().next().map(Extension::stem).unwrap_or(self.name)
	}

	/// The final extension.
	pub fn extension(self) -> Option<&'a str> {
		self.extensions().next().map(Extension::extension)
	}

	/// Iterate over all extensions.
	pub fn extensions(self) -> Extensions<'a> {
		Extensions::new(self.name)
	}
}

/// Iterators for use with pure paths.
pub mod iter {
	// These iterators could stand to be improved a lot.
	// Also a little unsafe would go a long way in simplifying things.

	use core::ops::Deref;
	/// Represents a single extension. E.g. `.tar.gz` the result of `.extension()`
	/// may be either start or end.
	#[derive(Debug, Copy, Clone)]
	pub struct Extension<'a> {
		file_name: &'a str,
		start: usize,
		end: usize,
	}
	impl<'a> Extension<'a> {
		fn new(file_name: &'a str) -> Self {
			let len = file_name.len();
			Self { file_name, start: len, end: len }
		}

		/// The current extension.
		pub fn extension(self) -> &'a str {
			&self.file_name[self.start + 1..self.end]
		}

		/// Split the file name into `(name, extension)` at this point.
		pub fn split_once(self) -> (&'a str, &'a str) {
			self.file_name.split_at(self.start)
		}

		/// The entire extension from this point. E.g. `tar.gz`.
		pub fn full_extension(self) -> &'a str {
			self.split_once().1
		}

		/// Get the filename without this extension.
		pub fn stem(self) -> &'a str {
			self.split_once().0
		}
	}

	/// Iterator over one or more file extensions.
	#[derive(Debug)]
	pub struct Extensions<'a> {
		current: Extension<'a>,
	}
	impl<'a> Extensions<'a> {
		pub fn new(name: &'a str) -> Self {
			Self { current: Extension::new(name) }
		}
	}
	impl<'a> Iterator for Extensions<'a> {
		type Item = Extension<'a>;
		fn next(&mut self) -> Option<Self::Item> {
			let name = self.current.stem();
			if let Some(position) = name.bytes().rposition(|b| b == b'.') {
				// Unix file names use the `.` prefix to mean a file should be hidden.
				// So we skip the first dot for consistency with both that and std.
				// However, this should perhaps be configurable in some way.
				if position == 0 {
					None
				} else {
					self.current.end = self.current.start;
					self.current.start = position;
					Some(self.current)
				}
			} else {
				None
			}
		}
	}

	/// Represents a component that's currently being iterated.
	#[derive(Debug, Clone, Copy)]
	pub struct Component<'a, const SEPARATOR: char> {
		path: &'a str,
		start: usize,
		end: usize,
		component: super::Component<'a, SEPARATOR>,
	}
	impl<'a, const SEPARATOR: char> Deref for Component<'a, SEPARATOR> {
		type Target = super::Component<'a, SEPARATOR>;
		fn deref(&self) -> &Self::Target {
			&self.component
		}
	}
	impl<'a, const SEPARATOR: char> Component<'a, SEPARATOR> {
		fn new(path: &'a str) -> Self {
			Self { path, start: 0, end: 0, component: super::Component::new_unchecked("") }
		}

		fn reverse_new(path: &'a str) -> Self {
			let len = path.len();
			Self { path, start: len, end: len, component: super::Component::new_unchecked("") }
		}

		/// Get the current component.
		pub fn component(self) -> super::Component<'a, SEPARATOR> {
			self.component
		}

		/// Get this component's parent path.
		pub fn parent(self) -> &'a super::PurePath<SEPARATOR> {
			self.split_once().0
		}

		/// Get the rest of the path, including this component.
		pub fn path(&self) -> &super::PurePath<SEPARATOR> {
			self.split_once().1
		}

		/// Returns the parent path and then the rest of the path including the
		/// current component.
		pub fn split_once(
			self,
		) -> (&'a super::PurePath<SEPARATOR>, &'a super::PurePath<SEPARATOR>) {
			let (a, b) = self.path.split_at(self.start);
			// Don't include the initial separator.
			let b = b.strip_suffix(SEPARATOR).unwrap_or(b);
			(super::PurePath::from_str_unchecked(a), super::PurePath::from_str_unchecked(b))
		}
	}

	/// Iterator over the components of a path.
	#[derive(Debug, Clone, Copy)]
	pub struct Components<'a, const SEPARATOR: char> {
		current: Option<Component<'a, SEPARATOR>>,
	}
	impl<'a, const SEPARATOR: char> Components<'a, SEPARATOR> {
		pub(super) fn new(path: &'a str) -> Self {
			if path.is_empty() {
				Self { current: None }
			} else {
				Self { current: Some(Component::new(path)) }
			}
		}
	}
	impl<'a, const SEPARATOR: char> Iterator for Components<'a, SEPARATOR> {
		type Item = Component<'a, SEPARATOR>;
		fn next(&mut self) -> Option<Self::Item> {
			let this = self.current.as_mut()?;
			let mut current = *this;
			current.start = current.end;
			if let Some(position) = current.path[current.start..].find(SEPARATOR) {
				current.end = current.start + position;
				current.component =
					super::Component::new_unchecked(&current.path[current.start..current.end]);
				this.end = current.end + SEPARATOR.len_utf8();
				Some(current)
			} else {
				current.end = current.path.len();
				current.component =
					super::Component::new_unchecked(&current.path[current.start..current.end]);
				self.current = None;
				Some(current)
			}
		}
	}

	#[derive(Debug, Clone, Copy)]
	pub struct Ancestors<'a, const SEPARATOR: char> {
		current: Option<Component<'a, SEPARATOR>>,
	}
	impl<'a, const SEPARATOR: char> Ancestors<'a, SEPARATOR> {
		pub(super) fn new(path: &'a str) -> Self {
			if path.is_empty() {
				Self { current: None }
			} else {
				Self { current: Some(Component::reverse_new(path)) }
			}
		}
	}
	impl<'a, const SEPARATOR: char> Iterator for Ancestors<'a, SEPARATOR> {
		type Item = Component<'a, SEPARATOR>;
		fn next(&mut self) -> Option<Self::Item> {
			let this = self.current.as_mut()?;
			this.end = this.start;
			if let Some(position) = this.path[..this.end].rfind(SEPARATOR) {
				this.start = position;
				this.component = super::Component::new_unchecked(
					&this.path[this.start + SEPARATOR.len_utf8()..this.end],
				);
				Some(*this)
			} else {
				let mut this = *this;
				this.start = 0;
				this.component = super::Component::new_unchecked(&this.path[this.start..this.end]);
				self.current = None;
				Some(this)
			}
		}
	}
}
use iter::*;
