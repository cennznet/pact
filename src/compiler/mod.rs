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

use crate::interpreter::{Comparator, Conjunction, OpCode};
use crate::parser::ast;
use crate::types::{Contract, DataTable, Numeric, PactType, StringLike};

use hashbrown::HashMap;

/// Indicates whether the source of a load is an `Input`
/// or stored on the compiled `DataTable`
#[derive(Clone, Copy)]
pub enum LoadSource {
    Input,
    DataTable,
}

/// A source for a subject for comparison
pub struct SubjectSource {
    pub load_source: LoadSource,
    pub index: u8,
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
                if idents.len() >= MAX_ENTRIES {
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
        let rhs_load = self.compile_subject(&assertion.rhs_subject)?;

        match (lhs_load.load_source.clone(), rhs_load.load_source.clone()) {
            (LoadSource::DataTable, LoadSource::DataTable) => {
                return Err(CompileErr::InvalidCompare)
            }
            (_, _) => {}
        }

        // Build and compile comparator
        let _ = OpCode::COMP(Comparator::from(&assertion.comparator)
            .apply_imperative(&assertion.imperative)
            .loads_from_subjects(lhs_load, rhs_load))
            .compile(&mut self.bytecode)?;

        // Handle conjunction if it exists
        if let Some((conjunctive, conjoined_assertion)) = &assertion.conjoined_assertion {
            let _ = OpCode::CONJ(Conjunction::from(conjunctive))
                .compile(&mut self.bytecode)?;
            self.compile_assertion(&*conjoined_assertion)?;
        }

        Ok(())
    }

    /// Compile a subject AST node
    fn compile_subject(&mut self, subject: &'a ast::Subject) -> Result<SubjectSource, CompileErr> {
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
                Ok(SubjectSource {
                    load_source: LoadSource::DataTable,
                    index: (self.data_table.len() as u8) - 1,
                })
            }
            ast::Subject::Identifier(ident) => {
                // Try lookup this var `ident` in the known input and user data tables
                if let Some(index) = self.input_var_index.get(ident) {
                    return Ok(SubjectSource {
                        load_source: LoadSource::Input,
                        index: *index,
                    });
                }
                if let Some(index) = self.user_var_index.get(ident) {
                    return Ok(SubjectSource {
                        load_source: LoadSource::DataTable,
                        index: *index,
                    });
                }
                Err(CompileErr::UndeclaredVar(ident.to_string()))
            }
        }
    }
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
