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
//! The pact bytecode interpreter
//!
use crate::types::PactType;

pub use crate::types::opcode::{
    Comparator, Conjunction, OpCode, OpComp, OpConj, OpIndices, OpLoad,
};

/// Interpret some pact byte code (`source`) with input data registers (`input_data`) and
/// user data registers (`user_data`).
/// Returns a boolean indicating whether the pact contract was validated or not,
/// An `InterpErr` is returned on a runtime error e.g. malformed byte code, missing data, invalid OpCode etc.
pub fn interpret(
    input_data: &[PactType],
    user_data: &[PactType],
    source: &[u8],
) -> Result<bool, InterpErr> {
    let mut interpreter = Interpreter::new(input_data, user_data);
    let mut scanner = source.iter();
    while let Some(op) = OpCode::parse(&mut scanner)? {
        match interpreter.interpret(op) {
            Err(InterpErr::Refused) => break,
            Err(err) => return Err(err),
            Ok(_) => {}
        }
    }

    match interpreter.state {
        State::AssertionTrue => Ok(true),
        State::Failed | State::AssertionFalse => Ok(false),
        // Any other state is an Unexpected end of input
        _invalid => Err(InterpErr::UnexpectedEOI("incomplete operation")),
    }
}

/// An interpreter error
#[derive(Debug, PartialEq)]
pub enum InterpErr {
    /// A comparison operator failed with incompatible types on LHS and RHS
    TypeMismatch,
    /// A comparison operator failed because it is not supported on the type
    BadTypeOperation,
    /// Unexpected end of input
    UnexpectedEOI(&'static str),
    /// Encountered an unexpected OpCode given the context
    UnexpectedOpCode(u8),
    /// Encountered an OpCode the interpreter does not support yet
    UnsupportedOpCode(&'static str),
    /// Encountered an invalid OpCode
    InvalidOpCode(u8),
    /// A referenced index in the data table does not exist
    MissingIndex(u8),
    /// Raised when trying to execute an OpCode from an interpreter which is in a failed state
    Refused,
}

/// Evaluate a comparator OpCode returning its result
fn eval_comparator(
    comparator: Comparator,
    lhs: &PactType,
    rhs: &PactType,
) -> Result<bool, InterpErr> {
    let value = match (lhs, rhs) {
        (PactType::Numeric(l), PactType::Numeric(r)) => match comparator.op {
            OpComp::EQ => Ok(l == r),
            OpComp::GT => Ok(l > r),
            OpComp::GTE => Ok(l >= r),
            _ => Err(InterpErr::BadTypeOperation),
        },
        (PactType::StringLike(l), PactType::StringLike(r)) => match comparator.op {
            OpComp::EQ => Ok(l == r),
            _ => Err(InterpErr::BadTypeOperation),
        },
        (PactType::List(_), _) => match comparator.op {
            _ => Err(InterpErr::BadTypeOperation),
        },
        (l, PactType::List(r)) => match comparator.op {
            OpComp::IN => Ok(r.contains(l)),
            _ => Err(InterpErr::BadTypeOperation),
        },
        _ => Err(InterpErr::TypeMismatch),
    }?;

    // Apply inversion if required
    if comparator.invert {
        Ok(!value)
    } else {
        Ok(value)
    }
}

/// Evaluate a conjunction OpCode given an LHS and RHS boolean
fn eval_conjunction(conjunction: &Conjunction, lhs: bool, rhs: bool) -> Result<bool, InterpErr> {
    let value = match conjunction.op {
        OpConj::AND => lhs & rhs,
        OpConj::OR => lhs | rhs,
        OpConj::XOR => lhs ^ rhs,
    };

    // Apply inversion if required
    if conjunction.invert {
        Ok(!value)
    } else {
        Ok(value)
    }
}

/// The pact interpreter
/// It evaluates `OpCode`s maintaining the state of the current contract execution
/// Uses the rust type system to encode state, see: https://hoverbear.org/2016/10/12/rust-state-machine-pattern/
/// States provide transformations into other valid states and failure cases.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Interpreter<'a> {
    state: State,
    input_data: &'a [PactType<'a>],
    user_data: &'a [PactType<'a>],
}

impl<'a> Interpreter<'a> {
    /// Return a new interpreter, ready for execution
    pub fn new(input_data: &'a [PactType<'a>], user_data: &'a [PactType<'a>]) -> Self {
        Interpreter {
            state: State::Initial,
            input_data,
            user_data,
        }
    }

    /// Executes a comparator OpCode
    /// This belongs to the interpreter state machine and will update state
    /// based on the outcome
    fn execute_comparator(&mut self, op: OpCode) -> Result<(), InterpErr> {
        match op {
            OpCode::COMP(comparator) => {
                // Gather left and right hand side values
                let lhs = self
                    .input_data
                    .get(comparator.indices.lhs as usize)
                    .ok_or(InterpErr::MissingIndex(comparator.indices.lhs))?;

                let rhs = match comparator.load {
                    OpLoad::INPUT_VS_USER => self
                        .user_data
                        .get(comparator.indices.rhs as usize)
                        .ok_or(InterpErr::MissingIndex(comparator.indices.rhs)),
                    OpLoad::INPUT_VS_INPUT => self
                        .input_data
                        .get(comparator.indices.rhs as usize)
                        .ok_or(InterpErr::MissingIndex(comparator.indices.rhs)),
                }?;

                let mut result = eval_comparator(comparator, &lhs, rhs)?;

                // Evaluate the conjunction if necessary
                match &self.state {
                    State::Conjunctive {
                        last_assertion,
                        conjunction,
                    } => {
                        result = eval_conjunction(conjunction, *last_assertion, result)?;
                    }
                    _ => {}
                };

                // The assertions and operations upto this point have all been collapsed into
                // a single boolean.
                if result {
                    self.state = State::AssertionTrue;
                } else {
                    self.state = State::AssertionFalse;
                };
                Ok(())
            }
            _ => Err(InterpErr::UnexpectedOpCode(op.into())),
        }
    }

    /// Interpreter state machine
    pub fn interpret(&mut self, op: OpCode) -> Result<(), InterpErr> {
        match &self.state {
            // First op code must be a comparator
            State::Initial => self.execute_comparator(op),
            State::AssertionTrue => match op {
                OpCode::COMP(_) => self.execute_comparator(op),
                OpCode::CONJ(conjunction) => {
                    self.state = State::Conjunctive {
                        last_assertion: true,
                        conjunction: conjunction,
                    };
                    Ok(())
                }
            },
            State::AssertionFalse => {
                match op {
                    // There is no continuation of the last assertion.
                    // This is now considered a failed clause, and hence the contract has failed
                    OpCode::COMP(_) => {
                        self.state = State::Failed;
                        Ok(())
                    }
                    // The conjunction will determine whether the contract has failed or succeeded
                    OpCode::CONJ(conjunction) => {
                        self.state = State::Conjunctive {
                            last_assertion: false,
                            conjunction: conjunction,
                        };
                        Ok(())
                    }
                }
            }
            State::Conjunctive {
                last_assertion: _,
                conjunction: _,
            } => {
                // A Conjunction must be followed by a comparator
                match op {
                    OpCode::COMP(_) => self.execute_comparator(op),
                    OpCode::CONJ(_) => {
                        return Err(InterpErr::UnexpectedOpCode(op.into()));
                    }
                }
            }
            State::Failed => Err(InterpErr::Refused),
        }
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
pub enum State {
    /// The initial interpreter state
    Initial,
    /// The last assertion evaluated as false
    AssertionFalse,
    /// The last assertion evaluated as true
    AssertionTrue,
    /// The last assertion was followed by a conjunction.
    /// The interpreter is awaiting the next OpCode as the RHS.
    Conjunctive {
        // The last assertion truthiness (LHS of conjunction)
        last_assertion: bool,
        // The conjunction to apply. <LHS> <conjunction> <RHS>
        conjunction: Conjunction,
    },
    /// The contract invariants were not maintained
    /// it has failed.
    Failed,
}
