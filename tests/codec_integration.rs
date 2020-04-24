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

//! Codec integration tests

#![cfg(test)]
use pact::interpreter::{Comparator, OpCode, OpComp, OpIndices, OpInvert, OpLoad};
use pact::types::{BinaryFormatErr, Contract, DataTable, Numeric, PactType, StringLike};

#[test]
fn contract_binary_format_codec() {
    let expected = Contract {
        data_table: DataTable::new(vec![
            PactType::Numeric(Numeric(111)),
            PactType::Numeric(Numeric(333)),
            PactType::StringLike(StringLike(b"testing")),
        ]),
        bytecode: [
            // EQ LD_INPUT(0) LD_USER(0)
            OpCode::COMP(Comparator {
                load: OpLoad::INPUT_VS_USER,
                op: OpComp::EQ,
                indices: OpIndices { lhs: 1, rhs: 0 },
                invert: OpInvert::NORMAL,
            })
            .into(),
            0x10,
            // EQ LD_INPUT(1) LD_USER(1)
            OpCode::COMP(Comparator {
                load: OpLoad::INPUT_VS_USER,
                op: OpComp::EQ,
                indices: OpIndices { lhs: 1, rhs: 1 },
                invert: OpInvert::NORMAL,
            })
            .into(),
            0x11,
        ]
        .to_vec(),
    };

    let mut buf: Vec<u8> = Vec::new();
    expected.encode(&mut buf);

    let result = Contract::decode(&buf).expect("it decodes");

    assert_eq!(result, expected);
}

#[test]
fn contract_binary_format_malformed_data_table() {
    let mut malformed_short: Vec<u8> = vec![0, 1];
    assert_eq!(
        Contract::decode(&mut malformed_short),
        Err(BinaryFormatErr::MalformedDataTable("missing type ID byte"))
    );

    let mut bad_type_id = vec![0, 0b1000_0000, 0b0000_0001, 0b0000_0000];
    assert_eq!(
        Contract::decode(&mut bad_type_id),
        Err(BinaryFormatErr::MalformedDataTable("unsupported type ID"))
    );

    let mut numeric_too_small = vec![0, 0b1000_0000, 0b1000_0000, 0b0100_0000, 0, 0];
    assert_eq!(
        Contract::decode(&mut numeric_too_small),
        Err(BinaryFormatErr::MalformedDataTable(
            "implementation only supports 64-bit numerics"
        ))
    );
}
