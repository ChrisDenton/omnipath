# Overview

## Win32 Paths

[Win32 paths](./Win32.md) are a living history of Microsoft OSes from DOS 1.0 through Windows 95/NT and modern Windows.

### Absolute Win32 paths

|Type|Examples|
|--|--|
|Drive|`C:\Windows`|
|<abbr title="Universal Naming Convention">UNC</abbr>|`\\server\share\`|
|Device|`\\.\PIPE\name`|`\??\PIPE\name`|
|Verbatim|`\\?\C:\Windows`<br>`\\?\UNC\server\share\`<br>`\\?\PIPE\name`|

### Relative Win32 paths

|Type|Examples|
|---|---|
|Path Relative|`file.ext`<br>`.\file.ext`<br>`..\file.ext`|
|Root Relative|`\file.ext`|
|Drive Relative|`D:file.ext`|

## Path character encoding

Paths are [UTF-16 strings](./Strings.md). Windows allows using other encodings (including UTF-8) but these are all lossily converted to and from UTF-16.

## Disallowed characters

[Filesystem drivers](./Filesystems.md) typically disallow the following characters in path components:

|Disallowed|Description|
|--|--|
|`\` `/`|Path seperators|
|`:`|Dos drive and NTFS file stream separator|
|`*` `?`|Wildcards|
|`<` `>` `"`|DOS wildcards|
|<code>\|</code>|Pipe|
|`NUL` to `US`|ASCII control codes; aka Unicode C0 control codes (U+0000 to U+001F inclusive). Note that `DEL` (U+007F) is allowed.|

Note that path separators and wildcards must be disallowed in normal filesystems otherwise some Win32 APIs will be unusable in some situations.

## Special Dos Device Names

For legacy reasons, some filenames may be interpreted as [DOS devices](./Special%20Dos%20Device%20Names.md). This means, for example the path "AUX" will be rewritten as `\\.\AUX`.

The following are special dos device names:

* `AUX`
* `CON`
* `CONIN$`
* `CONOUT$`
* `COM1`, `COM2`, `COM3`, `COM4`, `COM5`, `COM6`, `COM7`, `COM8`, `COM9`, `COM²`, `COM³`, `COM¹`
* `LPT1`, `LPT2`, `LPT3`, `LPT4`, `LPT5`, `LPT6`, `LPT7`, `LPT8`, `LPT9`, `LPT²`, `LPT³`, `LPT¹`
* `NUL`
* `PRN`

Note that these names are case-insensitive though canonically they're uppercase.

## Under the hood

Win32 paths are emulated on top of [NT kernel paths](./NT.md). An NT path looks similar to a Unix path, except for the directory separator. For example:

    \Device\HarddiskVolume2\directory\subdir\file.ext

These types of paths cannot be used directly in the Win32 API but can make themsevles apparent in other ways.
