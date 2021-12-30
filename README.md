# A Rust crate that provides random access to the unicode characters in a file

To provide slightly better performance than just scanning the string it maintains a buffer of line ending positions.
