# Special DOS Device names

In the Win32 namespace a path that matches the name of a special DOS device may be resolved to that device instead of to a file path. For example, that path:

    COM1

Will resolve to:

    \\.\COM1

Which becomes the kernel path:

    \??\COM1

These are the DOS device names that get the path replaced:

* `AUX`
* `CON`
* `CONIN$`
* `CONOUT$`
* `COM1`, `COM2`, `COM3`, `COM4`, `COM5`, `COM6`, `COM7`, `COM8`, `COM9`, `COM²`, `COM³`, `COM¹`
* `LPT1`, `LPT2`, `LPT3`, `LPT4`, `LPT5`, `LPT6`, `LPT7`, `LPT8`, `LPT9`, `LPT²`, `LPT³`, `LPT¹`
* `NUL`
* `PRN`

However the algorithm for matching device names is not as simple as a direct comparison and also depends on the OS version.

## Windows 11

Windows 11 greatly simplified how these device names are handled compared to earlier versions of Windows.

To test if a path matches a special dos device, it's as if the following steps were taken before comparing:

1. ASCII letters are uppercased
2. trailing dots (`.`) and spaces (` `) are removed

So `cOm1..  ..` is interpreted as `\\.\COM1` but `.\COM1` isn't.

The one remaining complication is the `NUL` device. If this appears in the filename (aka last component) of an absolute DOS drive or a relative path then the filename itself will be compared using the steps above. But this only happens if the parent directory actually exists thus it's as though every directory has a virtual `NUL` file.

So the following paths are interpreted as `\\.\NUL` if their parent directory exists:

    C:\path\to\nul

Again, this only applies to `NUL` so `C:\path\to\COM1` will be treated as a normal file path.

## Windows 10 and earlier

If a path is an absolute DOS drive or a relative path and if a filename (aka the final component) matches one of the special DOS device name then the path is ignored and replaced with that DOS device. For example:

    C:\path\to\COM1

Gets translated to:

    \\.\COM1


It's as if the following steps were applied to the file name before comparing:

1. ASCII letters are uppercased
2. anything after a `.` and the `.` itself are removed
3. any trailing spaces (` `) are stripped.

For example, these filenames are all interpreted as `\\.\COM1`:

* "<code>COM1.ext</code>"
* "<code>COM1 &nbsp; &nbsp; </code>"
* "<code>COM1 . .ext</code>"

When opening a file path such as `C:\Test\COM1`, it will only resolve to `\\.\COM1` if the parent directory `C:\Test` exists. Otherwise opening the file will fail with an invalid path error.
