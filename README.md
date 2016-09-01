# rust-bytestool
Compiler plugin to handle constant byte expressions: concatinating byte strings and calculating byte-size of byte string literals.

The compiler plugin allows you to handle byte (u8) arrays and combine them, for example to form constant network messages. The array may be immutable and formed at compilation time.

```rust
#![feature(asm)]
#![feature(plugin)]
#![plugin(bytestool)] // import macros byte_size_of and concat_bytes
#![feature(type_ascription)]

fn send_hello()
{
   let mesg_hello : &[u8; 1 + byte_size_of!(b"HELLO")] = concat_bytes!(b"HELLO", [23u8]);
   
   // send out the byte message via network
}
```

Note: byte strings are not terminated by null-character. The byte-size of b"0123" will be 4of size 4  (in C the string "0123" would be of size 5).

These macros can be used within macros itself.