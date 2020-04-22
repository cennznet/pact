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

use crate::interpreter::{Comparator, OpCode, OpComp, OpConj, OpIndices, OpInvert, OpLoad, OpType};
use crate::parser::ast;
use crate::types::{Contract, DataTable, Numeric, PactType, StringLike};

use hashbrown::HashMap;

pub enum LoadSource {
    Input,
    DataTable,
}

const MAX_ENTRIES: usize = 16;

/// Compilation error
#[derive(Debug, PartialEq)]
pub enum CompileErr {
    /// The identifier used is not declared
    UndeclaredVar(ast::Identifier),
    /// A parameter with the same identifier has already been declared
    Redeclared,
    InvalidListElement,
    /// Comparing user data table entries is not valid
    InvalidCompare,
    /// Data table is full
    DataTableFull,
    /// Too Many Input arguments
    TooManyInputs,
}

/// Compile a pact contract AST into bytecode
pub fn compile(ir: &[ast::Node]) -> Result<Contract, CompileErr> {
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
                if idents.len() > MAX_ENTRIES {
                    return Err(CompileErr::TooManyInputs);
                }
                for (index, ident) in idents.iter().enumerate() {
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
                    ast::Value::List(l) => {
                        let mut list = Vec::<PactType>::with_capacity(l.len());
                        for element in l {
                            list.push(match element {
                                ast::Value::Numeric(n) => PactType::Numeric(Numeric(*n)),
                                ast::Value::StringLike(s) => {
                                    PactType::StringLike(StringLike(s.as_bytes()))
                                }
                                _ => return Err(CompileErr::InvalidListElement),
                            })
                        }
                        PactType::List(list)
                    }
                };
                compiler.push_to_datatable(v)?;
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

/// A source for a subject for comparison
struct SubjectSource {
    load_source: LoadSource,
    index: u8,
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

    fn push_to_datatable(&mut self, value: PactType<'a>) -> Result<(), CompileErr> {
        if self.data_table.len() >= MAX_ENTRIES {
            Err(CompileErr::DataTableFull)
        } else {
            self.data_table.push(value);
            Ok(())
        }
    }

    /// Compile an assertion AST node
    fn compile_assertion(&mut self, assertion: &'a ast::Assertion) -> Result<(), CompileErr> {
        let lhs_load = self.compile_subject(&assertion.lhs_subject)?;
        let (comparator_op, invert) = compile_comparator(&assertion.imperative, &assertion.comparator)?;
        let rhs_load = self.compile_subject(&assertion.rhs_subject)?;

        // Determine the Load Order
        let (load, flip) = match lhs_load.load_source {
            LoadSource::Input => match rhs_load.load_source {
                LoadSource::Input => (OpLoad::INPUT_VS_INPUT, false),
                LoadSource::DataTable => (OpLoad::INPUT_VS_USER, false),
            },
            LoadSource::DataTable => match rhs_load.load_source {
                LoadSource::Input => (OpLoad::INPUT_VS_USER, true),
                _ => return Err(CompileErr::InvalidCompare),
            },
        };

        // Check whether we need to flip the load indices
        // to meet the load order
        let indices = if flip {
            OpIndices {
                lhs: rhs_load.index,
                rhs: lhs_load.index,
            }
        } else {
            OpIndices {
                lhs: lhs_load.index,
                rhs: rhs_load.index,
            }
        };

        // A flip means we need to redefine inequalities
        let (comparator_op, invert) = if flip {
            match comparator_op {
                OpComp::EQ => (comparator_op, invert),
                OpComp::IN => (comparator_op, invert),
                OpComp::GT => {
                    // Convert to LT
                    let invert = match invert {
                        OpInvert::NORMAL => OpInvert::NOT,
                        OpInvert::NOT => OpInvert::NORMAL,
                    };
                    (OpComp::GTE, invert)
                }
                OpComp::GTE => {
                    // Convert to LTE
                    let invert = match invert {
                        OpInvert::NORMAL => OpInvert::NOT,
                        OpInvert::NOT => OpInvert::NORMAL,
                    };
                    (OpComp::GT, invert)
                }
            }
        } else {
            (comparator_op, invert)
        };

        // Form the comparator opcode structure and push it out
        let op = OpCode {
            op_type: OpType::COMP(Comparator {
                load: load,
                op: comparator_op,
                indices: indices,
            }),
            invert: invert,
        };
        self.bytecode.push(op.into());
        self.bytecode.push(indices.into());

        // Handle conjunction if it exists
        if let Some((conjunctive, conjoined_assertion)) = &assertion.conjoined_assertion {
            self.bytecode
                .push(compile_conjunctive(&conjunctive)?.into());
            self.compile_assertion(&*conjoined_assertion)?;
        }

        Ok(())
    }

    /// Compile a subject AST node
    fn compile_subject(
        &mut self,
        subject: &'a ast::Subject,
    ) -> Result<SubjectSource, CompileErr> {
        // `subject` could be a literal value or an identifier
        // A literal value should be stored in the user data table
        // An identifier should have been declared or it is an error
        match subject {
            ast::Subject::Value(value) => {
                // convert ast::Value to PactType
                let v = match value {
                    ast::Value::Numeric(n) => PactType::Numeric(Numeric(*n)),
                    ast::Value::StringLike(s) => PactType::StringLike(StringLike(s.as_bytes())),
                    ast::Value::List(_) => panic!("Invalid subject"),
                };
                self.push_to_datatable(v)?;
                Ok( SubjectSource {
                    load_source: LoadSource::DataTable,
                    index: (self.data_table.len() as u8) - 1,
                })
            }
            ast::Subject::Identifier(ident) => {
                // Try lookup this var `ident` in the known input and user data tables
                if let Some(index) = self.input_var_index.get(ident) {
                    return Ok( SubjectSource {
                        load_source: LoadSource::Input,
                        index: *index,
                    });
                }
                if let Some(index) = self.user_var_index.get(ident) {
                    return Ok( SubjectSource {
                        load_source: LoadSource::DataTable,
                        index: *index,
                    });
                }
                Err(CompileErr::UndeclaredVar(ident.to_string()))
            }
        }
    }
}

/// Compile a conjunction AST node
fn compile_conjunctive(conjunctive: &ast::Conjunctive) -> Result<OpCode, CompileErr> {
    Ok(match conjunctive {
        ast::Conjunctive::And => OpCode {
            op_type: OpType::CONJ(OpConj::AND),
            invert: OpInvert::NORMAL,
        },
        ast::Conjunctive::Or => OpCode {
            op_type: OpType::CONJ(OpConj::OR),
            invert: OpInvert::NORMAL,
        },
    })
}

/// Compile a comparator AST node
fn compile_comparator(
    imperative: &ast::Imperative,
    comparator: &ast::Comparator,
) -> Result<(OpComp, OpInvert), CompileErr> {
    // Because of inequalities, the comparator and inversion
    // operations are tightly coupled.
    let invert: OpInvert = match imperative {
        ast::Imperative::MustBe => OpInvert::NORMAL,
        ast::Imperative::MustNotBe => OpInvert::NOT,
    };
    Ok(match comparator {
        ast::Comparator::Equal => (OpComp::EQ, invert),
        ast::Comparator::GreaterThan => (OpComp::GT, invert),
        ast::Comparator::GreaterThanOrEqual => (OpComp::GTE, invert),
        ast::Comparator::LessThan => {
            let invert = match invert {
                OpInvert::NORMAL => OpInvert::NOT,
                OpInvert::NOT => OpInvert::NORMAL,
            };
            (OpComp::GTE, invert)
        }
        ast::Comparator::LessThanOrEqual => {
            let invert = match invert {
                OpInvert::NORMAL => OpInvert::NOT,
                OpInvert::NOT => OpInvert::NORMAL,
            };
            (OpComp::GT, invert)
        }
        ast::Comparator::OneOf => (OpComp::IN, invert),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::BinaryFormatErr;

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
