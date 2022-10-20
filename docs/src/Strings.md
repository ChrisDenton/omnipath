# Strings

## Encoding

In Windows, all paths are treated as Unicode strings. However the Win32 API provides convinence functions to automatically convert the system encoding to UTF-16 (and vice versa). This helps to avoid the [Mojibake](https://en.wikipedia.org/wiki/Mojibake) problem by only having one canonical encoding. The UTF-16 conversion happens before everything else so interpreting paths only needs to operate on UTF-16 strings.

## `UnicodeString`

The NT kernel uses UTF-16 strings. Their definition is conceptually similar to Rust's `Vec<u16>`:

```rust
struct UnicodeString {
    length: u16,
    capacity: u16,
    buffer: *mut u16,
}
```

Note that these strings can contain nulls. However, if they do then they will not be useable by the Windows API.

Unlike Rust's `String`, the kernel will not check that the UnicodeString struct contains valid UTF-16. This means that it's possible for malicious applications to create file names with isolated surrogates (i.e. invalid Unicode).

## Windows API Strings

In the Win32 API there are generally two types of strings that applications can choose to use. Both are `NULL` terminated.

* Multibyte: `*mut u8`
* Wide: `*mut u16`.

Multibyte strings are in whichever encoding is set by the user or system. Windows will automatically convert to and from a UTF-16 `UnicodeString` as needed. If a Multibyte string contains bytes that are invalid for that encoding then they may be replaced when converting to UTF-16.

Recent versions of Windows also have the UTF-8 local encoding which, like other local encodings, is lossily converted to and from UTF-16.

Wide strings are UTF-16 and are put into a `UnicodeString` struct without being checked, except to get the length. So again it's possible for the string to contain invalid Unicode.
