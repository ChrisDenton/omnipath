# Filesystems

While the kernel allows almost anything in component names, filesystems may be more restrictive. For example, an [NT path](Object%20Manager.md) can include a component called `C:` but a filesystem may not allow you to create a directory with that name.

Microsoft's filesystem drivers will typically not allow the following characters in component names:

|Disallowed|Description|
|--|--|
|`\` `/`|Path seperators|
|`:`|Dos drive and NTFS file stream separator|
|`*` `?`|Wildcards|
|`<` `>` `"`|DOS wildcards|
|<code>\|</code>|Pipe|
|`NUL` to `US`|ASCII control codes; aka Unicode C0 control codes (U+0000 to U+001F inclusive). Note that `DEL` (U+007F) is allowed.|

Each component in a path is currently limited to 255 UTF-16 code units. However, it may not be safe to rely on this.

Filesystem paths may or may not be case sensitive. In Windows they are typically case insensitive but this cannot always be assumed. In some circumstances case sensitivity can even differ on a per directory basis.

While filesystems could be more relaxed about valid characters, path separators (`\/`) and wildcards (`*?<>"`) must be disallowed in normal filesystems. Otherwise some Win32 APIs will be unusable in some situations.

## File streams

The above disallowed characters applies to component names but NTFS understands an additional syntax: file streams. Each file (including directories) can have multiple streams of data. You can address them like so:

    file.ext:stream_name

Which is also equivalent to:

    file.ext:stream_name:$DATA

The stream name cannot contain a `NULL` (`0x0000`) or have the characters `\`, `/`, `:`. Like path components, it's limited to 255 UTF-16 code units.

The `$DATA` part of the stream identifier is a stream type. Valid types are assigned by Microsoft and always start with a `$`. If not specified, the type defaults to `$DATA`.

Directories also have a special directory stream type and will default to it if no stream name is given. For example:

    dir_name

Is equivalent to:

    dir_name:$I30:$INDEX_ALLOCATION

## Special filesystems

There are some special devices which accept paths but aren't true filesystem. For example, the `NUL` device will claim every path exists (even those that are usually invalid). The `PIPE` device simply treats paths strings. It does not have actual directories, although it does treat some prefixes specially (e.g. `LOCAL\`.
