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

//!
//! Primitive types in the pact interpreter.
//!
pub use crate::interpreter::type_cast::IntoPact;
use bit_reverse::ParallelReverse;
use std::vec::Vec;

/// A string-like type
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(PartialEq, PartialOrd, Clone)]
pub struct StringLike<'a>(pub &'a [u8]);

/// A numeric type
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(PartialEq, PartialOrd, Clone)]
pub struct Numeric(pub u64);

/// Over-arching pact type system
#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(Clone)]
pub enum PactType<'a> {
    StringLike(StringLike<'a>),
    Numeric(Numeric),
}

impl<'a> PactType<'a> {
    /// Encode the PactType into `buf`c
    pub fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            PactType::StringLike(s) => {
                buf.push(0);
                buf.push((s.0.len() as u8).swap_bits());
                buf.extend(s.0.iter());
            }
            PactType::Numeric(n) => {
                buf.push(1.swap_bits());
                // only supporting 64-bit numeric here.
                buf.push(8.swap_bits());
                for b in n.0.to_le_bytes().iter() {
                    buf.push(b.swap_bits())
                }
            }
        };
    }
    /// Decode a pact type from the given buffer
    /// Returns (decoded type, bytes read) or error on failure
    pub fn decode(buf: &'a [u8]) -> Result<(Self, usize), &'static str> {
        // Empty or too short (needs at least one header byte)
        if buf.len() <= 1 {
            return Err("too short");
        }
        match buf[0].swap_bits() {
            0 => {
                let read_len = 2usize + buf[1].swap_bits() as usize;
                let s = PactType::StringLike(StringLike(&buf[2..read_len]));
                Ok((s, read_len))
            }
            1 => {
                // only supporting 64-bit numeric here
                if (buf[2..].len() as u8) < 8 {
                    return Err("implmentation only supports 64-bit numerics");
                }
                let n = PactType::Numeric(Numeric(u64::from_le_bytes([
                    buf[2].swap_bits(),
                    buf[3].swap_bits(),
                    buf[4].swap_bits(),
                    buf[5].swap_bits(),
                    buf[6].swap_bits(),
                    buf[7].swap_bits(),
                    buf[8].swap_bits(),
                    buf[9].swap_bits(),
                ])));
                Ok((n, 10usize))
            }
            _ => Err("unsupported type ID"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_string_like() {
        let s = PactType::StringLike(StringLike(b"hello world"));
        let buf: &mut Vec<u8> = &mut Vec::new();
        s.encode(buf);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1].swap_bits(), 11);
        assert_eq!(&buf[2..], "hello world".as_bytes());
    }

    #[test]
    fn it_encodes_numeric() {
        let n = PactType::Numeric(Numeric(123));
        let buf: &mut Vec<u8> = &mut Vec::new();
        n.encode(buf);

        let mut expected: Vec<u8> = vec![1, 8, 123, 0, 0, 0, 0, 0, 0, 0];
        expected = expected.into_iter().map(|b| b.swap_bits()).collect(); // convert to LE bit orders
        assert_eq!(buf, &expected);
    }

    #[test]
    fn it_decodes_string_like() {
        let mut buf = vec![0, 11];
        buf = buf.into_iter().map(|b| b.swap_bits()).collect(); // convert to LE bit orders
        buf.extend("hello world".as_bytes());
        let (string_type, bytes_read) = PactType::decode(&buf).expect("it decodes");

        assert_eq!(
            string_type,
            PactType::StringLike(StringLike(b"hello world")),
        );

        assert_eq!(bytes_read, 13usize,);
    }

    #[test]
    fn it_decodes_numeric() {
        let mut encoded: Vec<u8> = vec![1, 8, 123, 0, 0, 0, 0, 0, 0, 0];
        encoded = encoded.into_iter().map(|b| b.swap_bits()).collect(); // convert to LE bit orders
        let (numeric_type, bytes_read) = PactType::decode(&encoded).expect("it decodes");

        assert_eq!(numeric_type, PactType::Numeric(Numeric(123)));
        assert_eq!(10usize, bytes_read,);
    }
}
