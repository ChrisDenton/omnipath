pub(crate) mod kind;
#[cfg(any(doc, all(windows, feature = "std")))]
mod sys;

#[cfg(any(doc, all(windows, feature = "std")))]
pub use sys::{resolve_prefix, WinPathExt};

pub use kind::{Win32Relative, WinPathKind};
