//! The public API is currently unstable so most of the actual string parsing
//! should go here.

use alloc::string::String;
use core::{
	fmt::{self, Write},
	iter::DoubleEndedIterator,
	ops::Deref,
};

/// A string path or component, not including any prefix.
#[repr(transparent)]
#[derive(Debug)]
pub struct StrPath {
	path: str,
}
impl StrPath {
	pub const fn from_str(str: &str) -> &Self {
		unsafe { &*(str as *const str as *const Self) }
	}

	pub const fn as_str(&self) -> &str {
		&self.path
	}

	pub fn win_display(
		&self,
		f: &mut fmt::Formatter<'_>,
		force_unix_separators: bool,
	) -> fmt::Result {
		if force_unix_separators {
			for component in self.verbatim_components() {
				f.write_str(component.as_str())?;
				f.write_char('/')?;
			}
			Ok(())
		} else {
			f.write_str(self.as_str())
		}
	}

	pub const fn is_empty(&self) -> bool {
		self.path.is_empty()
	}
	pub const fn len(&self) -> usize {
		self.path.len()
	}

	pub fn ends_with_char(&self, char: char) -> bool {
		self.path.ends_with(char)
	}

	pub fn verbatim_components(&self) -> impl Iterator<Item = &Self> {
		self.path.split('\\').map(Self::from_str)
	}

	pub fn reverse_verbatim_components(&self) -> impl Iterator<Item = &Self> {
		self.path.rsplit('\\').map(Self::from_str)
	}

	pub fn parent_verbatim(&self) -> Option<&Self> {
		self.path.rsplit_once('\\').map(|s| Self::from_str(s.0))
	}

	/// Iterate the ancestors of the path.
	///
	/// The value of skip will contain the remaining number of `..` components,
	/// once the iterator has finished,
	pub fn win32_ancestors<'a, 'b>(
		&'a self,
		skip: &'b mut usize,
	) -> impl Iterator<Item = &'a Self> + 'b
	where
		'a: 'b,
		'b: 'a,
	{
		let path = &self.path;
		// A path ending with a trailing `\` causes this iterator to emit an empty
		// component so that it is preserved.
		if path.ends_with(['\\', '/']) { [""].iter() } else { [].iter() }
			.copied()
			.chain(
				path.rsplit(['\\', '/'])
					// Ignore empty or dot components
					.filter(|&c| !c.is_empty() && c != ".")
					// Parse `..` components
					.filter(|&c| {
						if c == ".." {
							*skip += 1;
							false
						} else if *skip > 0 {
							*skip -= 1;
							false
						} else {
							true
						}
					})
					// Strip a single trailing dot but not two or more dots.
					.map(|c| {
						if c.ends_with(".") && !c.ends_with("..") {
							&c[..c.len() - 1]
						} else {
							c
						}
					}),
			)
			.map(Self::from_str)
	}

	/// Used for checking if a component can be round tripped through verbatim->win32->verbatim
	/// without changing the meaning of the path.
	///
	/// This assumes the component may be used as a file name (aka the last component of a path).
	/// It does not account for special dos device names.
	pub fn is_win32_safe(&self) -> bool {
		let path = Self::from_str(self.path.strip_suffix('\\').unwrap_or(&self.path));
		path.verbatim_components().all(Self::is_component_win32_safe)
	}

	pub fn is_component_win32_safe(&self) -> bool {
		let component = &self.path;
		!(component.is_empty()
			|| component.ends_with(['.', ' '])
			|| component.contains(['/', '\0']))
	}

	// Removes trailing dots and spaces ('.' and ' ')
	pub fn trim_filename(&self) -> &Self {
		Self::from_str(self.path.trim_end_matches(['.', ' ']))
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct StrPathBuffer {
	path: String,
}
impl StrPathBuffer {
	pub fn from_string_mut(str: &mut String) -> &mut Self {
		unsafe { &mut *(str as *mut String as *mut Self) }
	}
	pub fn as_str(&self) -> &str {
		&self.path
	}
	pub fn as_string(&self) -> &String {
		&self.path
	}
	pub fn as_string_mut(&mut self) -> &mut String {
		&mut self.path
	}
	pub fn push(&mut self, str: &StrPath) {
		self.path.push_str(str.as_str());
	}
	pub fn push_char(&mut self, char: char) {
		self.path.push(char);
	}
	pub fn push_component(&mut self, str: &StrPath) {
		if !self.is_empty() && !self.as_str().ends_with('\\') {
			self.push_char('\\');
		}
		self.push(str);
	}

	/// Remove the last character.
	pub fn pop(&mut self) -> Option<char> {
		self.path.pop()
	}

	pub fn pop_verbatim_component(&mut self) -> bool {
		if let Some(pos) = self.path.as_bytes().iter().rposition(|&b| b == b'\\') {
			self.path.truncate(pos);
			true
		} else {
			false
		}
	}
}
impl Deref for StrPathBuffer {
	type Target = StrPath;
	fn deref(&self) -> &Self::Target {
		StrPath::from_str(&self.path)
	}
}

/// Strip trailing `.` and ` ` (space) from a full path.
///
/// This will preserve trailing `.` and `..` components.
pub fn trim_full_path(path: &str) -> &str {
	if let Some(pos) = path.rfind(|b| b != '.' && b != ' ') {
		match path.split_at(pos + 1) {
			(trimmed, "." | "..") if trimmed.is_empty() || trimmed.ends_with(['\\', '/']) => path,
			(trimmed, _) => trimmed,
		}
	} else {
		path
	}
}
