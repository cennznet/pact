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
use crate::interpreter::OpCode;
use crate::parser::ast;

mod data_table;
pub use data_table::DataTable;

use bit_reverse::ParallelReverse;
use hashbrown::HashMap;

/// Compilation error
#[derive(Debug)]
pub enum CompileErr {
    /// The identifier used is not declared
    UndeclaredVar(ast::Identifier),
    /// A parameter with the same identifier has already been declared
    Redeclared,
}

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

/// Compile a pact contract AST into bytecode
pub fn compile<'a>(ir: &'a [ast::Node]) -> Result<Contract<'a>, CompileErr> {
    // 1. Semantically verify the AST
    //     - Duplicate var definition
    //     - Missing var definition
    //     - Comparisons between incompatible var types
    // 2. Move user-defined vars into a data section
    // 3. Replace var identifiers with data indexes
    // 4. Replace input param identifiers with data indexes
    let mut compiler = Compiler::new();

    for node in ir.iter() {
        match node {
            ast::Node::InputDeclaration(idents) => {
                for (index, ident) in idents.into_iter().enumerate() {
                    compiler
                        .input_var_index
                        .insert(ident.to_string(), index as u8);
                }
            }
            ast::Node::Clause(assertion) => {
                compiler.compile_assertion(&assertion)?;
            }
            ast::Node::Definition(identifier, value) => {
                if compiler.input_var_index.contains_key(identifier) {
                    return Err(CompileErr::Redeclared);
                }
                let previous = compiler
                    .user_var_index
                    .insert(identifier.to_string(), compiler.user_var_index.len() as u8);
                if previous.is_some() {
                    return Err(CompileErr::Redeclared);
                }

                // convert ast::Value to PactType
                let v = match value {
                    ast::Value::Numeric(n) => PactType::Numeric(Numeric(*n)),
                    ast::Value::StringLike(s) => PactType::StringLike(StringLike(s.as_bytes())),
                };
                compiler.data_table.push(v)
            }
        }
    }

    Ok(Contract {
        data_table: DataTable::new(compiler.data_table),
        bytecode: compiler.bytecode,
    })
}

/// A pact compiler
struct Compiler<'a> {
    data_table: Vec<PactType<'a>>,
    bytecode: Vec<u8>,
    // Intermediate store for user var definitions (identity, u8 ordered index)
    input_var_index: HashMap<String, u8>,
    // Intermediate store for input var ordering (identity, u8 ordered index)
    user_var_index: HashMap<String, u8>,
}

impl<'a> Compiler<'a> {
    /// Create a new Compiler
    fn new() -> Self {
        Self {
            data_table: Default::default(),
            bytecode: Default::default(),
            input_var_index: Default::default(),
            user_var_index: Default::default(),
        }
    }

    /// Compile an assertion AST node
    fn compile_assertion(&mut self, assertion: &'a ast::Assertion) -> Result<(), CompileErr> {
        let comparator_op = compile_comparator(&assertion.2)?;
        self.bytecode.push(comparator_op.into());

        let lhs_load_op = self.compile_subject(&assertion.0)?;
        match lhs_load_op {
            OpCode::LD_INPUT(index) | OpCode::LD_USER(index) => {
                self.bytecode.push(lhs_load_op.into());
                self.bytecode.push(index);
            }
            _ => panic!("unreachable"),
        };

        // TODO: Imperative is ignored for now. In future it will set/flip the truthiness of the comparator op

        let rhs_load_op = self.compile_subject(&assertion.3)?;
        match rhs_load_op {
            OpCode::LD_INPUT(index) | OpCode::LD_USER(index) => {
                self.bytecode.push(rhs_load_op.into());
                self.bytecode.push(index);
            }
            _ => panic!("unreachable"),
        };

        if let Some((conjunctive, conjoined_assertion)) = &assertion.4 {
            self.bytecode
                .push(compile_conjunctive(&conjunctive)?.into());
            self.compile_assertion(&*conjoined_assertion)?;
        }

        Ok(())
    }

    /// Compile a subject AST node
    fn compile_subject(&mut self, subject: &'a ast::Subject) -> Result<OpCode, CompileErr> {
        // `subject` could be a literal value or an indentifier
        // A literal value should be stored in the user data table
        // An identifier should have been declared or it is an error
        match subject {
            ast::Subject::Value(value) => {
                // convert ast::Value to PactType
                let v = match value {
                    ast::Value::Numeric(n) => PactType::Numeric(Numeric(*n)),
                    ast::Value::StringLike(s) => PactType::StringLike(StringLike(s.as_bytes())),
                };
                self.data_table.push(v);
                Ok(OpCode::LD_USER((self.data_table.len() as u8) - 1))
            }
            ast::Subject::Identifier(ident) => {
                // Try lookup this var `ident` in the known input and user data tables
                if let Some(index) = self.input_var_index.get(ident) {
                    return Ok(OpCode::LD_INPUT(*index));
                }
                if let Some(index) = self.user_var_index.get(ident) {
                    return Ok(OpCode::LD_USER(*index));
                }
                Err(CompileErr::UndeclaredVar(ident.to_string()))
            }
        }
    }
}

/// Compile a conjunction AST node
fn compile_conjunctive(conjunctive: &ast::Conjunctive) -> Result<OpCode, CompileErr> {
    Ok(match conjunctive {
        ast::Conjunctive::And => OpCode::AND,
        ast::Conjunctive::Or => OpCode::OR,
    }
    .into())
}

/// Compile a comparator AST node
fn compile_comparator(comparator: &ast::Comparator) -> Result<OpCode, CompileErr> {
    Ok(match comparator {
        ast::Comparator::Equal => OpCode::EQ,
        ast::Comparator::GreaterThan => OpCode::GT,
        ast::Comparator::GreaterThanOrEqual => OpCode::GTE,
        ast::Comparator::LessThan => OpCode::LT,
        ast::Comparator::LessThanOrEqual => OpCode::LTE,
    }
    .into())
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
