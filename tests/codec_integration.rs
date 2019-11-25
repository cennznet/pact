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

#![cfg(test)]
use pact::contract::{BinaryFormatErr, Contract, DataTable};
use pact::interpreter::OpCode;
use pact::types::{Numeric, PactType, StringLike};

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
            OpCode::EQ.into(),
            0,
            0,
            1,
            0,
            // EQ LD_INPUT(1) LD_USER(1)
            OpCode::EQ.into(),
            0,
            1,
            1,
            1,
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
        Err(BinaryFormatErr::MalformedDataTable("too short"))
    );

    let mut bad_type_id = vec![0, 0b1000_0000, 0b0000_0001, 0b0000_0001];
    assert_eq!(
        Contract::decode(&mut bad_type_id),
        Err(BinaryFormatErr::MalformedDataTable("unsupported type ID"))
    );

    let mut numeric_too_large = vec![0, 0b1000_0000, 0b1000_0000, 0b0000_1111];
    assert_eq!(
        Contract::decode(&mut numeric_too_large),
        Err(BinaryFormatErr::MalformedDataTable(
            "implmentation only supports 64-bit numerics"
        ))
    );
}
