// Temporary while this code is being fleshed out.
#![allow(dead_code)]
use core::str;

use crate::util;

// .:?/\a
// &['\\', '/', '.', '?', ':', 'T', '£', '三', '😍']

/// Parse the prefix from a path.
pub struct ParsedUtf8Path<'a> {
    path: &'a str,
    kind: WinPathKind,
    prefix_len: usize,
}
impl<'a> ParsedUtf8Path<'a> {
    /// Parse a UTF-8 string into a prefix and subpath..
    pub fn from_utf8(path: &'a str) -> ParsedUtf8Path<'a> {
        let (kind, len) = WinPathKind::from_str_with_len(path);
        Self {
            path,
            kind,
            prefix_len: match kind {
                WinPathKind::Unc => len + str_unc_prefix_len(&path[len..]),
                _ => len,
            },
        }
    }

    /// Get the original, unparsed path.
    pub fn as_utf8(&self) -> &str {
        self.path
    }

    /// Get the type of path.
    pub const fn kind(&self) -> WinPathKind {
        self.kind
    }

    /// Normalize the kind
    pub fn normalized_str_kind(&self) -> NormalizedStrKind {
        match self.kind() {
            WinPathKind::DriveRelative(_) => {
                let mut buffer = [0; 4];
                buffer[..self.prefix_len].copy_from_slice(&self.path.as_bytes()[..self.prefix_len]);
                NormalizedStrKind { buffer, len: self.prefix_len }
            }
            WinPathKind::Drive(_) => {
                let mut buffer = [0; 4];
                buffer[..self.prefix_len].copy_from_slice(&self.path.as_bytes()[..self.prefix_len]);
                buffer[self.prefix_len - 1] = b'\\';
                NormalizedStrKind { buffer, len: self.prefix_len }
            }
            WinPathKind::Verbatim => NormalizedStrKind { buffer: *br"\\?\", len: 4 },
            WinPathKind::Device => {
                // Preserves the `.` or `?`.
                // Maybe we should normalize this as `.` even if that's not what the OS does.
                let mut buffer = [b'\\'; 4];
                buffer[2] = self.path.as_bytes()[2];
                NormalizedStrKind { buffer, len: 4 }
            }
            WinPathKind::CurrentDirectoryRelative => NormalizedStrKind { buffer: [0; 4], len: 0 },
            WinPathKind::RootRelative => NormalizedStrKind { buffer: [b'\\', 0, 0, 0], len: 1 },
            WinPathKind::Unc => NormalizedStrKind { buffer: [b'\\', b'\\', 0, 0], len: 2 },
        }
    }

    /// Returns the (prefix, subpath) pair.
    pub fn parts<'b>(&'b self) -> (&'a str, &'a str)
    where
        'a: 'b,
    {
        self.path.split_at(self.prefix_len)
    }
}

pub struct NormalizedStrKind {
    buffer: [u8; 4],
    len: usize,
}
impl NormalizedStrKind {
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.buffer[..self.len]).unwrap()
    }
}

/// Parse the server and share name from the path.
///
/// This assumes the leading `\\` has already be parsed.
fn str_unc_prefix_len(path: &str) -> usize {
    let mut iter = path.as_bytes().iter();
    match iter.position(|&c| c == b'\\' || c == b'/') {
        Some(pos) => iter.position(|&c| c == b'\\' || c == b'/').map(|n| pos + n + 1),
        None => None,
    }
    .unwrap_or(path.len())
}

/// Windows path type.
///
/// This does not do any validation so parsing the kind will never fail,
/// even for broken or invalid paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinPathKind {
    /// A traditional drive path such as `C:\`, `R:\`, etc.
    Drive(u16),
    /// A path to a network directory such as `\\server\share\`.
    Unc,
    /// A device path such as `\\.\COM1`.
    Device,
    /// A path that's relative to the current directory.
    CurrentDirectoryRelative,

    /// A path that is passed to the NT kernel without parsing, except to change
    /// the prefix.
    ///
    /// These start with `\\?\`, avoid DOS path length limits and can contain
    /// paths that are otherwise illegal.
    Verbatim,

    /// A DOS drive relative path (e.g. `C:file`).
    DriveRelative(u16),
    /// A DOS root relative path (e.g. `\file`).
    ///
    /// Note that some Windows APIs can return paths that are relative to a
    /// specified drive. These may start with a `\` but should be joined to the
    /// drive path instead of being treated like a DOS `RootRelative` path.
    RootRelative,
}

impl WinPathKind {
    /// Split the path into `WinPathKind` and the rest of the path.
    ///
    /// Note that this only splits off the smallest part needed to identify the
    /// path type. E.g. the UNC path `\\server\share\file.txt` will be split
    /// as `(WinPathKind::Unc, "server\share\file.txt")`.
    pub const fn split_str(path: &str) -> (Self, &str) {
        let (kind, len) = Self::from_str_with_len(path);
        // SAFETY: Splitting the str only happens after an ASCII character.
        let rest = unsafe { util::trim_start_str(path, len) };
        (kind, rest)
    }

    pub(crate) const fn from_str_with_len(path: &str) -> (Self, usize) {
        let kind = Self::from_str(path);
        let len = match kind {
            Self::Drive(_) | Self::DriveRelative(_) => {
                kind.utf16_len() - 1 + (util::utf8_len(path.as_bytes()[0]) as usize)
            }
            _ => kind.utf16_len(),
        };
        (kind, len)
    }

    /// Examine the path prefix to find the type of the path given.
    pub const fn from_str(path: &str) -> Self {
        let bytes = path.as_bytes();

        if bytes.is_empty() {
            return WinPathKind::CurrentDirectoryRelative;
        }

        // If the path starts with `\\?\` then it's a verbatim path.
        // Note that this is an exact match. `//?/` is not a verbatim path.
        if let [b'\\', b'\\', b'?', b'\\', ..] = bytes {
            return Self::Verbatim;
        }
        if is_verbatim_str(path) {
            return Self::Verbatim;
        }

        match util::utf8_len(bytes[0]) {
            // If the first Unicode scalar would need more than one UTF-16 code unit
            // then this must be a relative path because it won't match any prefix.
            4.. => Self::CurrentDirectoryRelative,
            // If this first Unicode scalar is not ascii then this can only be
            // Drive, DriveRelative or Relative.
            n @ 2.. => {
                match_pattern! {
                    util::trim_start(bytes, n);
                    [':', /, ..] => Self::Drive(util::bmp_utf8_to_utf16(bytes)),
                    [':', ..] => Self::DriveRelative(util::bmp_utf8_to_utf16(bytes)),
                    _ => Self::CurrentDirectoryRelative
                }
            }
            // If this first Unicode scalar is ascii then it could be any type.
            // Warning: The order of these matches is super important.
            // You can't take a pattern out of this context without modification.
            _ => match_pattern! {
                bytes;
                // `\\.\` | `\\?\`
                [/, /, '.', /, ..] => Self::Device,
                // `\\`
                [/, /, ..] => Self::Unc,
                // `\`
                [/, ..] => Self::RootRelative,
                // `C:\`
                [_, ':', /, ..] => Self::Drive(bytes[0] as u16),
                // `C:`
                [_, ':', ..] => Self::DriveRelative(bytes[0] as u16),
                // Anything else
                _ => Self::CurrentDirectoryRelative
            },
        }
    }

    /// Is the path absolute. Being absolute means it doesn't need to be joined
    /// to a base path (e.g. the current directory, or a drive current directory)
    pub const fn is_absolute(self) -> bool {
        matches!(self, Self::Drive(_) | Self::Unc | Self::Device | Self::Verbatim)
    }

    /// Returns the relative path kind or `None` for absolute paths.
    pub const fn as_relative(self) -> Option<Win32Relative> {
        Win32Relative::from_kind(self)
    }

    /// Is the path one of the weird ones from DOS.
    ///
    /// These should probably be considered invalid if given from a configuration file
    /// but you may want to support them if given through command line arguments
    /// (e.g. because they come from the command prompt or a bat file).
    pub const fn is_legacy_relative(self) -> bool {
        matches!(self, Self::DriveRelative(_) | Self::RootRelative)
    }

    /// The number of UTF-16 code units that make up the path kind.
    pub const fn utf16_len(self) -> usize {
        match self {
            Self::Drive(_) => r"C:\".len(),
            Self::Unc => r"\\".len(),
            Self::Device => r"\\.\".len(),
            Self::CurrentDirectoryRelative => "".len(),
            Self::Verbatim => r"\\?\".len(),
            Self::DriveRelative(_) => "C:".len(),
            Self::RootRelative => r"\".len(),
        }
    }

    /// The number of UTF-8 code units that make up the path kind.
    pub const fn utf8_len(self) -> usize {
        const fn drive_utf8_len(drive: u16) -> usize {
            if drive > 0x7F {
                2
            } else {
                1
            }
        }
        match self {
            Self::Drive(drive) => drive_utf8_len(drive) + r":\".len(),
            Self::Unc => r"\\".len(),
            Self::Device => r"\\.\".len(),
            Self::CurrentDirectoryRelative => "".len(),
            Self::Verbatim => r"\\?\".len(),
            Self::DriveRelative(drive) => drive_utf8_len(drive) + ":".len(),
            Self::RootRelative => r"\".len(),
        }
    }
}

/// The type of relative path.
#[derive(Debug, Clone, Copy)]
pub enum Win32Relative {
    CurrentDirectory,
    DriveRelative(u16),
    Root,
}
impl Win32Relative {
    pub const fn from_kind(kind: WinPathKind) -> Option<Self> {
        match kind {
            WinPathKind::CurrentDirectoryRelative => Some(Self::CurrentDirectory),
            WinPathKind::DriveRelative(drive) => Some(Self::DriveRelative(drive)),
            WinPathKind::RootRelative => Some(Self::Root),
            _ => None,
        }
    }

    /// Is the path one of the weird ones from DOS.
    ///
    /// These should probably be considered invalid if given from a configuration file
    /// but you may want to support them if given through command line arguments
    /// (e.g. because they come from the command prompt or a bat file).
    pub const fn is_legacy_relative(self) -> bool {
        matches!(self, Self::DriveRelative(_) | Self::Root)
    }
}

/// The type of non-verbatim absolute path.
#[derive(Debug, Clone, Copy)]
pub enum Win32Absolute {
    Drive(u16),
    Unc,
    Device,
}
impl Win32Absolute {
    /// Verbatim paths will return `None`.
    pub const fn from_kind(kind: WinPathKind) -> Option<Self> {
        match kind {
            WinPathKind::Drive(drive) => Some(Self::Drive(drive)),
            WinPathKind::Unc => Some(Self::Unc),
            WinPathKind::Device => Some(Self::Device),
            _ => None,
        }
    }

    /// Get the Win32 type of a verbatim path.
    pub(crate) const fn from_verbatim_str(path: &str) -> Result<(Self, &str), ()> {
        let verbatim = match VerbatimStr::new(path) {
            Ok(verbatim) => verbatim,
            Err(e) => return Err(e),
        };
        let kind = verbatim.win32_kind();
        let rest = match kind {
            Win32Absolute::Unc => unsafe { util::trim_start_str(verbatim.path, "UNC".len()) },
            _ => verbatim.path,
        };
        Ok((kind, rest))
    }
}

pub struct VerbatimStr<'a> {
    path: &'a str,
}
impl<'a> VerbatimStr<'a> {
    const fn new(path: &'a str) -> Result<Self, ()> {
        match WinPathKind::split_str(path) {
            (WinPathKind::Verbatim, rest) => Ok(Self { path: rest }),
            _ => Err(()),
        }
    }
    const fn win32_kind(&self) -> Win32Absolute {
        // C:\, \\.\, \\
        match self.path.as_bytes() {
            // UNC\
            // Canonically `UNC` is uppercase but maybe we should ignore case here.
            [b'U', b'N', b'C', b'\\', ..] | [b'U', b'N', b'C'] => Win32Absolute::Unc,
            // C:\
            [d, b':', b'\\', ..] | [d, b':'] => Win32Absolute::Drive(*d as u16),
            [d1, d2, b':', b'\\', ..] | [d1, d2, b':'] => {
                let drive = util::bmp_utf8_to_utf16(&[*d1, *d2]);
                Win32Absolute::Drive(drive)
            }
            // Anything else is used as a device path.
            _ => Win32Absolute::Device,
        }
    }
}

#[inline]
pub const fn is_verbatim_str(path: &str) -> bool {
    matches!(path.as_bytes(), [b'\\', b'\\', b'?', b'\\', ..])
}
