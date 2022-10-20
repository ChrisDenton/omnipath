# Details

This section contains everything you ever wanted to know about Windows paths but were afraid to ask. Note that this discusses internal details that may be subject to change. It is intended to document the current state of path parsing in Windows and so not every detail can be relied on to always be true.

I'll start with NT kernel paths. These aren't usually used directly from user space but I promise they're important to fully understanding Win32 paths.
