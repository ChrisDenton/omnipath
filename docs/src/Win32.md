# Win32 Paths

The Win32 API is built as a layer on top of the NT kernel. It implements an API that was originally built for those familiar with Win16 and DOS so it doesn't directly deal with NT paths. Instead it converts Win32 paths to NT paths before calling the kernel.

Essentially Win32 paths are a user-space compatibility layer.

### Absolute Win32 paths

All absolute paths start with a root. On *nix the root is `/`. For the NT kernel it's `\`. In contrast, Win32 has four types of root and they're all longer than one character.

* `C:\`, `D:\`, `E:\`, etc. The first letter is a (case insensitive) drive letter that can be any ascii letter from `A` to `Z`.
* `\\server\share\` where `server` is the name of the server and `share` is the name of the shared directory. It is used to access a shared directory on a server therefore you must always specifiy both a server name and share name.
* `\\.\`. These are typically used to access devices other than drives or server shares (e.g. named pipes). So they are not usually filesystem paths.
* `\\?\`. These can be used to access any type of device.

The following table shows each type and an example of how the Win32 root is converted to a kernel path.

|Type|Win32 path|Kernel path|
|--|--|--|
|Drive|`C:\Windows`|`\??\C:\Windows`|
|<abbr title="Universal Naming Convention">UNC</abbr>|`\\server\share\file`|`\??\UNC\server\share\file`|
|Device|`\\.\PIPE\name`|`\??\PIPE\name`|
|Verbatim|`\\?\C:\Windows`<br>`\\?\UNC\server\share\file`<br>`\\?\PIPE\name`|`\??\C:\Windows`<br>`\??\UNC\server\share\file`<br>`\??\PIPE\name`|

From the table above it looks like device paths and verbatim paths work the same way. However, that's only because I left off a column: the namespace. The namespace determines what happens to the part of the path after the root. 

|Type|Namespace|Example|
|---|---|---|
|Drive|Win32|`C:\Windows`|
|<abbr title="Universal Naming Convention">UNC</abbr>|Win32|`\\server\share\file`|
|Device|Win32|`\\.\PIPE\name`|
|Verbatim|NT|`\\?\C:\Windows`<br>`\\?\UNC\server\share\file`<br>`\\?\PIPE\name`|

The next two sections will explain the effects the namespace has.

### NT namespace

Paths in the NT namespace are passed almost directly to the kernel without any transformations or substitutions.

The only Win32 paths in the NT namespace are verbatim paths (i.e. those that start with `\\?\`). When converting a verbatim path to a kernel path, all that happens is the root `\\?\` is changed to the kernel path `\??\`. The rest of the path is left untouched. See [NT Kernel Paths](./NT.md) for more on kernel paths.

Note that this is the only way to use kernel paths in the Win32 API. If you start a path with `\??\` or `\Device\` then it can have very different results.

### Win32 namespace

This section applies to all Win32 paths except for verbatim paths (those that start with `\\?\`).

When converting a Win32 path to a kernel path there are additional transformations and restrictions that are applied to DOS drive paths, UNC paths and Device paths. Some of these transformations are useful while others are an unfortunate holdover from DOS or early Windows.

Win32 namespaced paths are restricted to a length less than 260 UTF-16 code units. This restriction can be lifted on newer versions of Windows 10 but it requires both the user and the application to opt in.

When paths are in this namespace, one of two transformations may happen:

* If the path is a special DOS device name then a device path is returned. See [Special Dos Device Names](./Special%20Dos%20Device%20Names.md) for details.
* Otherwise the following transformations are applied:
   * First, all occurences of `/` are changed to `\`.
   * All path components consisting of only a single `.` are removed.
   * A sequence containing more than one `\` is replaced with a single `\`. E.g. `\\\` is collapsed to `\`.
   * All `..` path components will be removed along with their parent component. The Win32 root (e.g. `C:\`, `\\server\share`, `\\.\`) will never be removed.
   * If a component name ends with a `.` then the final `.` is removed, unless another `.` comes before it. So `dir.` becomes `dir` but `dir..` remains as it is. I'm sure there's a reason for this.
   * For the filename only (aka the last component), all trailing dots and spaces are stripped.

For example, this:

    C:/path////../../../to/.////file.. ..

Is changed to:

    C:\to\file

Which becomes the kernel path:

    \??\C:\to\file

This transformation all happens without touching the filesystem.

### Relative Win32 paths

Relative paths are usually resolved relative to the current directory. The current directory is a global mutable value that stores an absolute Win32 path to an existing directory. The current directory only supports DOS drive paths (e.g. `C:\`) and UNC paths (e.g. `\\server\share`). Using any other path type when setting the current directory is liable to break relative paths therefore verbatim paths (`\\?\`) should not be used.

There are three categories of relative Win32 paths.

|Type|Examples|
|---|---|
|Path Relative|`file.ext`<br>`.\file.ext`<br>`..\file.ext`|
|Root Relative|`\file.ext`|
|Drive Relative|`D:file.ext`|

Although Path Relative forms come in three flavours there are really only two. `file.txt` is interpreted exactly the same way as `.\file.txt` (see Win32 namespace). However, the `.\` prefix can help to avoid ambiguities introduced by drive relative paths.

Drive Relative paths are interpreted as being relative to the specified drive's current directory (note: usually only the command prompt has per drive current directories). Root relative are relative to the root of the current directory.

Drive Relative and Root Relative paths should be avoided whenever possible. Developers and users rarely understand how they're resolved so their results can be surprising. Additionally the Drive Relative paths syntax introduces ambiguity with file streams.