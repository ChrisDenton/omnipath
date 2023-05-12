A path handling library for Rust. See the [api docs](https://docs.rs/omnipath/latest/omnipath/)!

# Quick tour

The [`sys_absolute`](https://docs.rs/omnipath/0.1.6/omnipath/fn.sys_absolute.html) function is similar to [`std::fs::canonicalize`](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)
but doesn't require accessing the filesystem.

```rust
// Normalizes the path and makes it absolute.
// On Windows platforms this will use `\` as the path separator.
let absolute = omnipath::sys_absolute("path/to/.//file".as_ref());
```

The [`sys_canonicalize`](https://docs.rs/omnipath/0.1.6/omnipath/fn.sys_canonicalize.html) function is almost the same [`std::fs::canonicalize`](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)
except that it will try to return an non-verbatim path on Windows.

```rust
// On Windows this returns r"C:\path\to\file" instead of `\\?\C:\path\file`
let canonical = omnipath::sys_absolute(r"C:\path\to\file".as_ref());
```

## Platform-specific functions

The traits [PosixPathExt](https://docs.rs/omnipath/0.1.6/omnipath/posix/trait.PosixPathExt.html) and
[WinPathExt](https://docs.rs/omnipath/0.1.6/omnipath/windows/trait.WinPathExt.html) provide platform
specific extension traits for [`std::path::Path`](https://doc.rust-lang.org/std/path/struct.Path.html). For example, on Windows they allow [converting
to a user path](https://docs.rs/omnipath/0.1.6/omnipath/windows/trait.WinPathExt.html#tymethod.to_winuser_path)
(useful for displaying a path to the user without the `\\?\` part)
or [as a verbatim path](https://docs.rs/omnipath/0.1.6/omnipath/windows/trait.WinPathExt.html#tymethod.to_verbatim)
