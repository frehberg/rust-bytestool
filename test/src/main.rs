#![feature(asm)]
#![feature(plugin)]
#![plugin(bytestool)]
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
fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_byte_size_of() {
        assert_eq!(byte_size_of!(b"012345"), 6); // rust strings without NULL-termination
        assert_eq!(byte_size_of!(b"A"), 1); // rust strings without NULL-termination
        assert_eq!(byte_size_of!(b""), 0);
        assert_eq!(byte_size_of!(b"\x00"), 4); // escaped chars not supported yet
        assert_eq!(byte_size_of!([0u8, 1u8]), 2); // u8 array
    }

    #[test]
    fn test_concat_bytes() {
        assert_eq!(concat_bytes!(b"0123", b"45"), b"012345");
        assert_eq!(concat_bytes!(b"0123", b"45"),
                   &[48u8, 49u8, 50u8, 51u8, 52u8, 53u8]);
        assert_eq!(concat_bytes!(b"0123", [52u8, 53u8]), b"012345");

        assert_eq!(concat_bytes!([0u8], b"AA", [0u8]), &[0u8, 65u8, 65u8, 0u8]);

        let const_bytes: &[u8; 4] = concat_bytes!([0u8], b"AA", [0u8]);
        assert_eq!(const_bytes, &[0u8, 65u8, 65u8, 0u8]);

        let const_bytes: &[u8; byte_size_of!([0u8, 65u8, 65u8, 0u8])] =
            concat_bytes!([0u8], b"AA", [0u8]);
        assert_eq!(const_bytes, &[0u8, 65u8, 65u8, 0u8]);

        let const_bytes: &[u8; byte_size_of!([65u8, 65u8]) + 2] =
            concat_bytes!([0u8], b"AA", [0u8]);
        assert_eq!(const_bytes, &[0u8, 65u8, 65u8, 0u8]);
    }


    #[test]
    fn test_in_macro() {
        let assembled = assemble!(b"0123", b"45");
        assert_eq!(assembled, &[48u8, 49u8, 50u8, 51u8, 52u8, 53u8]);

        let assembled = assemble!([48u8, 49u8, 50u8, 51u8], [52u8, 53u8]);
        assert_eq!(assembled, &[48u8, 49u8, 50u8, 51u8, 52u8, 53u8]);

    }
}
