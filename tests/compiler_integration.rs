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

//! Compiler integration tests

#![cfg(test)]
use pact::compiler::{self};
use pact::interpreter;
use pact::parser;
use pact::types::{Numeric, PactType, StringLike};

#[test]
fn it_compiles() {
    let ast = parser::parse(
        "
          given parameters $a,$b,$user
          define $trusted as [\"Rick Astley\", \"bob\"]
          $a must be less than or equal to 123 and \"hello world\" must not be equal to $b
          $user must be one of $trusted
        ",
    )
    .unwrap();

    let contract = compiler::compile(&ast).unwrap();
    println!("Data Table: {:?}", contract.data_table);
    println!("Bytecode: {:?}", contract.bytecode);

    // The manually crafted input table
    // In normal execution, this contains the transaction arguments
    let input_table = &[
        PactType::Numeric(Numeric(5)),
        PactType::StringLike(StringLike("hello friend".as_bytes())),
        PactType::StringLike(StringLike("Rick Astley".as_bytes())),
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
