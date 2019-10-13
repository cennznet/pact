use crate::types::{Numeric, PactType, StringLike};
use std::any::Any;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum PactConversionErr {
    NegativeInteger,
    Overflow,
    UnknownType,
}

/// A catch-all conversion trait which tries to turn any given `value` into the implementing type
trait AnyTryInto<'a>: Sized {
    fn any_try_into(value: &'a dyn Any) -> Result<Self, PactConversionErr>;
}

/// A default implementation to return UnknownType err by default using specialization feature
impl<'a, T> AnyTryInto<'a> for T {
    default fn any_try_into(_value: &'a dyn Any) -> Result<Self, PactConversionErr> {
        Err(PactConversionErr::UnknownType)
    }
}

impl<'a> AnyTryInto<'a> for PactType<'a> {
    fn any_try_into(value: &'a dyn Any) -> Result<PactType<'a>, PactConversionErr> {
        if let Some(number) = value.downcast_ref::<i8>() {
            if *number < 0 {
                return Err(PactConversionErr::NegativeInteger);
            }
            if let Ok(n) = u64::try_from(*number) {
                return Ok(PactType::Numeric(Numeric(n)));
            }
        }
        // ... the above repeated for all integer types

        if let Some(string) = value.downcast_ref::<&str>() {
            return Ok(PactType::StringLike(StringLike(&*string.as_bytes())));
        }
        // ... the above repeated for all string-like types

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
            PactType::any_try_into(&0_i8),
            Ok(PactType::Numeric(Numeric(0))),
        );
        assert_eq!(
            PactType::any_try_into(&0_i128),
            Err(PactConversionErr::UnknownType),
        );
    }

    #[test]
    fn it_converts_string() {
        assert_eq!(
            PactType::any_try_into(&"test"),
            Ok(PactType::StringLike(StringLike(b"test"))),
        );
    }
}
