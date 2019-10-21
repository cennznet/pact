// Copyright (C) 2019 Centrality Investments Limited
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::interpreter::types::{Numeric, PactType, StringLike};

/// A blanket trait for conversion into PactType
pub trait IntoPact<'a, I> {
    fn into_pact(self) -> PactType<'a>;
}

/// Dummy structs
struct Numbers;
struct Strings;

/// Impl for all types that implement lossless conversion into u64
impl<'a, T: Into<u64> + Copy> IntoPact<'a, &Numbers> for T {
    fn into_pact(self) -> PactType<'a> {
        PactType::Numeric(Numeric(Into::<u64>::into(self)))
    }
}

/// Impl for all types that can be converted to &[u8]
impl<'a, T: AsRef<[u8]> + ?Sized> IntoPact<'a, &Strings> for &'a T {
    fn into_pact(self) -> PactType<'a> {
        PactType::StringLike(StringLike(self.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_numeric() {
        let tests = vec![
            (0_u8.into_pact(), PactType::Numeric(Numeric(0))),
            (1_u16.into_pact(), PactType::Numeric(Numeric(1))),
            (2_u32.into_pact(), PactType::Numeric(Numeric(2))),
            (3_u64.into_pact(), PactType::Numeric(Numeric(3))),
        ];
        for (lhs, rhs) in tests {
            assert_eq!(lhs, rhs);
        }
    }

    #[test]
    fn it_converts_string_like() {
        assert_eq!(
            "test".into_pact(),
            PactType::StringLike(StringLike(b"test")),
        );

        let v: Vec<u8> = vec![116, 101, 115, 116];
        assert_eq!(v.into_pact(), PactType::StringLike(StringLike(b"test")),);

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
            (h32.into_pact(), PactType::StringLike(StringLike(h32))),
            (h64.into_pact(), PactType::StringLike(StringLike(h64))),
            (h128.into_pact(), PactType::StringLike(StringLike(h128))),
            (h160.into_pact(), PactType::StringLike(StringLike(h160))),
            (h256.into_pact(), PactType::StringLike(StringLike(h256))),
            (h264.into_pact(), PactType::StringLike(StringLike(h264))),
            (h512.into_pact(), PactType::StringLike(StringLike(h512))),
            (h520.into_pact(), PactType::StringLike(StringLike(h520))),
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
            (n32.into_pact(), PactType::Numeric(Numeric(10))),
            (n64.into_pact(), PactType::Numeric(Numeric(20))),
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
            (s1.into_pact(), PactType::StringLike(StringLike(b"test1"))),
            (s2.into_pact(), PactType::StringLike(StringLike(b"test2"))),
            (s3.into_pact(), PactType::StringLike(StringLike(b"test3"))),
        ];
        for (lhs, rhs) in tests {
            assert_eq!(lhs, rhs);
        }
    }
}
