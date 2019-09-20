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
use pact::compiler::{self};
use pact::interpreter;
use pact::parser;
use pact::types::{Numeric, PactType, StringLike};

#[test]
fn it_compiles() {
    let ast = parser::parse(
        "
          given parameters $a,$b
          $a must be less than or equal to 123 and $b must be equal to \"hello world\"
        ",
    )
    .unwrap();

    let contract = compiler::compile(&ast).unwrap();
    println!("Data Table: {:?}", contract.data_table);
    println!("Bytecode: {:?}", contract.bytecode);

    // The manually crafted input table
    let input_table = &[
        PactType::Numeric(Numeric(5)),
        PactType::StringLike(StringLike("hello world".as_bytes())),
    ];
    println!("Input Table: {:?}", input_table);

    let result = interpreter::interpret(
        input_table,
        &contract.data_table.as_ref(),
        &contract.bytecode,
    );

    println!("Result: {:?}", result);
    assert!(result.unwrap());
}
