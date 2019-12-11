// Copyright 2019 Centrality Investments Limited
// This file is part of Pact.
//
// Licensed under the LGPL, Version 3.0 (the "License");
// you may not use this file except in compliance with the License.
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// You should have received a copy of the GNU General Public License
// along with Pact. If not, see:
//   <https://centrality.ai/licenses/gplv3.txt>
//   <https://centrality.ai/licenses/lgplv3.txt>

//!
//! Types in the pact interpreter aka "PactType"s
//!
use alloc::vec::Vec;
use bit_reverse::ParallelReverse;

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
        // Check type header bytes
        match buf.len() {
            0 => return Err("missing type ID byte"),
            1 => return Err("missing type length byte"),
            _ => (),
        };

        // 1 byte type ID + 1 byte length gives 2 offset
        let read_offset = 2_usize;

        // Read type ID byte
        match buf[0].swap_bits() {
            0 => {
                // Read length byte
                let data_length = buf[1].swap_bits() as usize;
                if data_length > buf[read_offset..].len() {
                    return Err("type length > buffer length");
                }
                let read_length = read_offset + data_length;
                let s = PactType::StringLike(StringLike(&buf[read_offset..read_length]));
                Ok((s, read_length))
            }
            1 => {
                let data_length = buf[1].swap_bits() as usize;
                if data_length > 8 {
                    return Err("implementation only supports 64-bit numerics");
                }
                if buf[read_offset..].len() < 8 {
                    return Err("type length > buffer length");
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

    #[test]
    fn it_fails_with_missing_type_id() {
        assert_eq!(PactType::decode(&[]), Err("missing type ID byte"));
    }

    #[test]
    fn it_fails_with_missing_type_length() {
        assert_eq!(PactType::decode(&[0]), Err("missing type length byte"));
    }

    #[test]
    #[should_panic(expected = "type length > buffer length")]
    fn it_fails_with_short_string_like() {
        PactType::decode(&[0, 11]).unwrap();
    }

    #[test]
    #[should_panic(expected = "implementation only supports 64-bit numerics")]
    fn it_fails_with_u128_numeric() {
        PactType::decode(&[1.swap_bits(), 16.swap_bits()]).unwrap();
    }
}
