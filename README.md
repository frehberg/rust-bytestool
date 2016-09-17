# rust-bytestool
Compiler plugin to handle constant byte expressions: concatinating byte strings and calculating byte-size of byte string literals.

The compiler plugin allows you to handle byte (u8) arrays and combine them, for example to form constant network messages. The array may be immutable and formed at compilation time.

example: Cargo.toml dependency
```init
[dependencies]
bytestool = "0.2.0"
```

example src/main.rs: forming a constant message to be sent over network
```rust
#![feature(asm)]
#![feature(plugin)]
#![plugin(bytestool)] // import macros byte_size_of and concat_bytes
#![feature(type_ascription)]

macro_rules! build_const_mesg {
    ($bstr1:expr, $bstr2:expr) => {{
            const LEN1 : usize = byte_size_of!($bstr1);
            const LEN2 : usize = byte_size_of!($bstr2);
            let result : &[u8; LEN1 + LEN2] = concat_bytes!($bstr1, $bstr2);
            result
    }};
}

fn send_hello()
{
   let mesg_hello = build_const_mesg!(b"HELLO", [23u8, 10u8, 10u8, 0u8] );

   // send out the byte message via network
}

fn send_bye()
{
   let mesg_bye   = build_const_mesg!(b"BYE",   [23u8, 10u8, 10u8, 0u8] );

   // send out the byte message via network
}
```

Note: byte strings are not terminated by null-character. The byte-size of b"0123" will be 4of size 4  (in C the string "0123" would be of size 5).

These macros can be used within macros itself.
