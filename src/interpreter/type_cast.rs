use crate::interpreter::types::{Numeric, PactType, StringLike};
use core::any::Any;
use core::convert::TryFrom;
use std::string::String;

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
        // TODO: refactor the below repetiion using macros
        // Unsigned integer type casting into PactType
        if let Some(number) = value.downcast_ref::<u8>() {
            if let Ok(n) = u64::try_from(*number) {
                return Ok(PactType::Numeric(Numeric(n)));
            }
        }
        if let Some(number) = value.downcast_ref::<u16>() {
            if let Ok(n) = u64::try_from(*number) {
                return Ok(PactType::Numeric(Numeric(n)));
            }
        }
        if let Some(number) = value.downcast_ref::<u32>() {
            if let Ok(n) = u64::try_from(*number) {
                return Ok(PactType::Numeric(Numeric(n)));
            }
        }
        if let Some(number) = value.downcast_ref::<u64>() {
            return Ok(PactType::Numeric(Numeric(*number)));
        }
        if let Some(number) = value.downcast_ref::<u128>() {
            if *number > core::u64::MAX as u128 {
                return Err(PactConversionErr::Overflow);
            }
            if let Ok(n) = u64::try_from(*number) {
                return Ok(PactType::Numeric(Numeric(n)));
            }
        }

        // String-like type casting into PactType
        if let Some(string) = value.downcast_ref::<&str>() {
            return Ok(PactType::StringLike(StringLike(&*string.as_bytes())));
        }
        if let Some(string) = value.downcast_ref::<String>() {
            return Ok(PactType::StringLike(StringLike(string.as_bytes())));
        }

        // Unhandled Type
        Err(PactConversionErr::UnknownType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_numeric() {
        assert_eq!(
            PactType::any_try_into(&0_u8),
            Ok(PactType::Numeric(Numeric(0))),
        );
        assert_eq!(
            PactType::any_try_into(&1_u128),
            Ok(PactType::Numeric(Numeric(1))),
        );

        // Assertion for overflow
        assert_eq!(
            PactType::any_try_into(&(core::u64::MAX as u128 + 2)),
            Err(PactConversionErr::Overflow),
        );
    }

    #[test]
    fn it_converts_string() {
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
    }
}
