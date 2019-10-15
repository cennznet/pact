use crate::interpreter::types::{Numeric, PactType, StringLike};
use core::any::Any;
use core::convert::TryFrom;
use std::string::String;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum PactConversionErr {
    Overflow,
    UnknownType,
}

/// A catch-all conversion trait which tries to turn any given `value` into the implementing type
pub trait AnyTryInto<'a>: Sized {
    fn any_try_into(value: &'a dyn Any) -> Result<Self, PactConversionErr>;
}

/// AnyTryInto implementation for PactType
impl<'a> AnyTryInto<'a> for PactType<'a> {
    fn any_try_into(value: &'a dyn Any) -> Result<PactType<'a>, PactConversionErr> {
        // TODO: refactor the below repetition using macros
        // Unsigned integer type casting into PactType
        if let Some(number) = value.downcast_ref::<u8>() {
            return Ok(PactType::Numeric(Numeric(*number as u64)));
        }
        if let Some(number) = value.downcast_ref::<u16>() {
            return Ok(PactType::Numeric(Numeric(*number as u64)));
        }
        if let Some(number) = value.downcast_ref::<u32>() {
            return Ok(PactType::Numeric(Numeric(*number as u64)));
        }
        if let Some(number) = value.downcast_ref::<u64>() {
            return Ok(PactType::Numeric(Numeric(*number)));
        }
        if let Some(number) = value.downcast_ref::<u128>() {
            return Ok(PactType::Numeric(Numeric(
                u64::try_from(*number).map_err(|_| PactConversionErr::Overflow)?,
            )));
        }

        // String-like type casting into PactType
        if let Some(string) = value.downcast_ref::<&str>() {
            return Ok(PactType::StringLike(StringLike(&*string.as_bytes())));
        }
        if let Some(string) = value.downcast_ref::<String>() {
            return Ok(PactType::StringLike(StringLike(string.as_bytes())));
        }
        if let Some(string) = value.downcast_ref::<Vec<u8>>() {
            return Ok(PactType::StringLike(StringLike(&*string)));
        }

        // Fixed hash type casting into PactType
        if let Some(string) = value.downcast_ref::<[u8; 4]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H32
        }
        if let Some(string) = value.downcast_ref::<[u8; 8]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H64
        }
        if let Some(string) = value.downcast_ref::<[u8; 16]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H128
        }
        if let Some(string) = value.downcast_ref::<[u8; 20]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H160
        }
        if let Some(string) = value.downcast_ref::<[u8; 32]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H256
        }
        if let Some(string) = value.downcast_ref::<[u8; 33]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H264
        }
        if let Some(string) = value.downcast_ref::<[u8; 64]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H512
        }
        if let Some(string) = value.downcast_ref::<[u8; 65]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H520
        }
        if let Some(string) = value.downcast_ref::<[u8; 128]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H1024
        }
        if let Some(string) = value.downcast_ref::<[u8; 256]>() {
            return Ok(PactType::StringLike(StringLike(&*string))); // H2048
        }

        // Unhandled Type
        Err(PactConversionErr::UnknownType)
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_numeric() {
        let tests = vec![
            (PactType::any_try_into(&0_u8),   Ok(PactType::Numeric(Numeric(0)))),
            (PactType::any_try_into(&1_u16),  Ok(PactType::Numeric(Numeric(1)))),
            (PactType::any_try_into(&2_u32),  Ok(PactType::Numeric(Numeric(2)))),
            (PactType::any_try_into(&3_u64),  Ok(PactType::Numeric(Numeric(3)))),
            (PactType::any_try_into(&4_u128), Ok(PactType::Numeric(Numeric(4)))),
        ];
        for (lhs, rhs) in tests {
            assert_eq!(lhs, rhs);
        }

        // Assertion for overflow
        assert_eq!(
            PactType::any_try_into(&(core::u64::MAX as u128 + 2)),
            Err(PactConversionErr::Overflow),
        );
    }

    #[test]
    fn it_converts_string_like() {
        assert_eq!(
            PactType::any_try_into(&"test"),
            Ok(PactType::StringLike(StringLike(b"test"))),
        );
        assert_eq!(
            PactType::any_try_into(&'a'.to_string()),
            Ok(PactType::StringLike(StringLike(b"a"))),
        );
        assert_eq!(
            PactType::any_try_into(&"test".to_string()),
            Ok(PactType::StringLike(StringLike(b"test"))),
        );

        let v: Vec<u8> = vec![116, 101, 115, 116];
        assert_eq!(
            PactType::any_try_into(&v),
            Ok(PactType::StringLike(StringLike(b"test")))
        );

        // Assertion for fixed hash types
        let h32 = b"0x01";
        let h64 = b"0x012345";
        let h128 = b"0x01234567891011";
        let h160 = b"0x012345678910111213";
        let h256 = b"0x012345678910111213141516171819";
        let h264 = b"0x0123456789101112131415161718192";
        let h512 = b"0x01234567891011121314151617181920212223242526272829303132333435";
        let h520 = b"0x012345678910111213141516171819202122232425262728293031323334353";

        let tests = vec![
            (PactType::any_try_into(h32),  Ok(PactType::StringLike(StringLike(h32)))),
            (PactType::any_try_into(h64),  Ok(PactType::StringLike(StringLike(h64)))),
            (PactType::any_try_into(h128), Ok(PactType::StringLike(StringLike(h128)))),
            (PactType::any_try_into(h160), Ok(PactType::StringLike(StringLike(h160)))),
            (PactType::any_try_into(h256), Ok(PactType::StringLike(StringLike(h256)))),
            (PactType::any_try_into(h264), Ok(PactType::StringLike(StringLike(h264)))),
            (PactType::any_try_into(h512), Ok(PactType::StringLike(StringLike(h512)))),
            (PactType::any_try_into(h520), Ok(PactType::StringLike(StringLike(h520)))),
        ];
        for (lhs, rhs) in tests {
            assert_eq!(lhs, rhs);
        }
    }

    #[test]
    fn it_converts_numeric_associated_types() {
        trait Foo {
            type Number32;
            type Number64;
        }

        struct Bar;

        impl Foo for Bar {
            type Number32 = u32;
            type Number64 = u64;
        }

        let n32: <Bar as Foo>::Number32 = 10u32;
        let n64: <Bar as Foo>::Number64 = 20u64;

        let tests = vec![
            (PactType::any_try_into(&n32), Ok(PactType::Numeric(Numeric(10)))),
            (PactType::any_try_into(&n64), Ok(PactType::Numeric(Numeric(20)))),
        ];
        for (lhs, rhs) in tests {
            assert_eq!(lhs, rhs);
        }
    }

    #[test]
    fn it_converts_string_like_associated_types() {
        trait Foo {
            type Ref;
            type Vec;
            type Str;
        }

        struct Bar;

        impl Foo for Bar {
            type Ref = &'static str;
            type Vec = Vec<u8>;
            type Str = String;
        }

        let s1: <Bar as Foo>::Ref = "test1";
        let s2: <Bar as Foo>::Vec = vec![116, 101, 115, 116, 50];
        let s3: <Bar as Foo>::Str = "test3".to_string();

        let tests = vec![
            (PactType::any_try_into(&s1), Ok(PactType::StringLike(StringLike(b"test1")))),
            (PactType::any_try_into(&s2), Ok(PactType::StringLike(StringLike(b"test2")))),
            (PactType::any_try_into(&s3), Ok(PactType::StringLike(StringLike(b"test3")))),
        ];
        for (lhs, rhs) in tests {
            assert_eq!(lhs, rhs);
        }
    }
}
