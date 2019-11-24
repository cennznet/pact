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

pub mod data_table;
use bit_reverse::ParallelReverse;
pub use data_table::DataTable;

#[derive(Debug, PartialEq)]
/// A binary format error
pub enum BinaryFormatErr {
    /// Version mismatch
    UnsupportedVersion,
    /// DataTable is invalid
    MalformedDataTable(&'static str),
    // The buffer is to short to be valid
    TooShort,
}

/// A pact contract
/// It has byte code and an accompanying data section
#[derive(Debug, PartialEq)]
pub struct Contract<'a> {
    pub data_table: DataTable<'a>,
    pub bytecode: Vec<u8>,
}

impl<'a> Contract<'a> {
    /// Encode the contract as v0 binary format into `buf`
    pub fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(0); // binary format version: `0`
        self.data_table.encode(buf);
        buf.extend(self.bytecode.clone());
    }
    /// Decode a pact contract from v0 binary format
    pub fn decode(buf: &'a [u8]) -> Result<Self, BinaryFormatErr> {
        if buf.len() < 2 {
            println!("len: {:?}", buf.len());
            return Err(BinaryFormatErr::TooShort);
        }
        if buf[0].swap_bits() != 0 {
            return Err(BinaryFormatErr::UnsupportedVersion);
        }
        let (data_table, offset) =
            DataTable::decode(&buf[1..]).map_err(|err| BinaryFormatErr::MalformedDataTable(err))?;
        let bytecode = buf[1usize + offset..].to_vec();
        Ok(Self {
            data_table,
            bytecode,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contract_binary_format_unsupported_version() {
        assert_eq!(
            Contract::decode(&[1, 0]),
            Err(BinaryFormatErr::UnsupportedVersion)
        );
    }

    #[test]
    fn contract_binary_format_too_short() {
        assert_eq!(Contract::decode(&[0]), Err(BinaryFormatErr::TooShort));
    }
}
