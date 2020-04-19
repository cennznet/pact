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
use pact::compiler::{self, CompileErr};
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

#[test]
fn it_fails_with_a_large_datatable_from_definitions() {
    let ast = parser::parse(
        "
          given parameters $a
          define $X0 as 0
          define $X1 as 1
          define $X2 as 2
          define $X3 as 3
          define $X4 as 4
          define $X5 as 5
          define $X6 as 6
          define $X7 as 7
          define $X8 as 8
          define $X9 as 9
          define $XA as 10
          define $XB as 11
          define $XC as 12
          define $XD as 13
          define $XE as 14
          define $XF as 15
          define $XG as 16
          $a must be less than or equal to $X0
        ",
    )
    .unwrap();
    assert_eq!(compiler::compile(&ast), Err(CompileErr::DataTableFull));
}

#[test]
fn it_fails_with_a_large_datatable_from_values() {
    let ast = parser::parse(
        "
          given parameters $a
          $a must be less than or equal to 0
          $a must be less than or equal to 1
          $a must be less than or equal to 2
          $a must be less than or equal to 3
          $a must be less than or equal to 4
          $a must be less than or equal to 5
          $a must be less than or equal to 6
          $a must be less than or equal to 7
          $a must be less than or equal to 8
          $a must be less than or equal to 9
          $a must be less than or equal to 10
          $a must be less than or equal to 11
          $a must be less than or equal to 12
          $a must be less than or equal to 13
          $a must be less than or equal to 14
          $a must be less than or equal to 15
          $a must be less than or equal to 16
        ",
    )
    .unwrap();
    assert_eq!(compiler::compile(&ast), Err(CompileErr::DataTableFull));
}

#[test]
fn it_fails_with_too_many_inputs() {
    let ast = parser::parse(
        "
          given parameters $x0, $x1, $x2, $x3, $x4, $x5, $x6, $x7, $x8, $x9, $xa, $xb, $xc, $xd, $xe, $xf, $xg
          $xf must be less than or equal to 16
        ",
    ).unwrap();
    assert_eq!(compiler::compile(&ast), Err(CompileErr::TooManyInputs));
}
