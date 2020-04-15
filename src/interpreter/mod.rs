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

// OpCode masks
const OP_MASK: u8 = 0b0011_1111;

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

/// A pact instruction code
///
/// Big Endian OpCodes
/// - 6 bit OpCode index
/// - 2 bit reserved
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpCode {
    /// Load an input var at index into the next free operand register
    LD_INPUT(u8),
    /// Load a data var at index into the next free operand register
    LD_USER(u8),
    /// Compute a logical and between A and the next comparator OpCode
    AND,
    /// Compute an inclusive or between A and the next comparator OpCode
    OR,
    /// Compute an exclusive or between A and the next comparator OpCode
    XOR,
    /// i == j
    EQ,
    /// i > j
    GT,
    /// i >= j
    GTE,
    /// i ∈ J
    IN,
    /// i < j
    LT,
    /// i <= j
    LTE,
}

impl OpCode {
    /// Return whether this OpCode is a load or not
    pub fn is_load(self) -> bool {
        match self {
            OpCode::LD_INPUT(_) | OpCode::LD_USER(_) => true,
            _ => false,
        }
    }
    /// Return whether this OpCode is a comparator or not
    pub fn is_comparator(self) -> bool {
        match self {
            OpCode::EQ | OpCode::GT | OpCode::GTE | OpCode::IN | OpCode::LT | OpCode::LTE => true,
            _ => false,
        }
    }
    /// Return whether this OpCode is a conjunction or not
    pub fn is_conjunction(self) -> bool {
        match self {
            OpCode::AND | OpCode::OR | OpCode::XOR => true,
            _ => false,
        }
    }

    /// Return the next OpCode by parsing an input byte stream
    pub fn parse(stream: &mut dyn Iterator<Item = &u8>) -> Result<Option<Self>, InterpErr> {
        let op_index = stream.next();
        if op_index.is_none() {
            // This is a valid EOI
            return Ok(None);
        }

        match op_index.unwrap() & OP_MASK {
            0 => {
                if let Some(index) = stream.next() {
                    Ok(Some(OpCode::LD_INPUT(*index)))
                } else {
                    Err(InterpErr::UnexpectedEOI("expected input index"))
                }
            }
            1 => {
                if let Some(index) = stream.next() {
                    Ok(Some(OpCode::LD_USER(*index)))
                } else {
                    Err(InterpErr::UnexpectedEOI("expected data index"))
                }
            }
            2 => Ok(Some(OpCode::AND)),
            3 => Ok(Some(OpCode::OR)),
            4 => Ok(Some(OpCode::XOR)),
            5 => Ok(Some(OpCode::EQ)),
            6 => Ok(Some(OpCode::GT)),
            7 => Ok(Some(OpCode::GTE)),
            8 => Ok(Some(OpCode::IN)),
            9 => Ok(Some(OpCode::LT)),
            10 => Ok(Some(OpCode::LTE)),
            _invalid => Err(InterpErr::InvalidOpCode(_invalid)),
        }
    }
}

/// Convert an OpCode into its u8 index.
/// It does not encode any following parameters
impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            OpCode::LD_INPUT(_) => 0,
            OpCode::LD_USER(_) => 1,
            OpCode::AND => 2,
            OpCode::OR => 3,
            OpCode::XOR => 4,
            OpCode::EQ => 5,
            OpCode::GT => 6,
            OpCode::GTE => 7,
            OpCode::IN => 8,
            OpCode::LT => 9,
            OpCode::LTE => 10,
        }
    }
}

/// Evaluate a comparator OpCode returning its result
fn eval_comparator(op: OpCode, lhs: &PactType, rhs: &PactType) -> Result<bool, InterpErr> {
    if !op.is_comparator() {
        return Err(InterpErr::UnexpectedOpCode(0));
    }

    match (lhs, rhs) {
        (PactType::Numeric(l), PactType::Numeric(r)) => match op {
            OpCode::EQ => Ok(l == r),
            OpCode::GT => Ok(l > r),
            OpCode::GTE => Ok(l >= r),
            OpCode::LT => Ok(l < r),
            OpCode::LTE => Ok(l <= r),
            _ => Err(InterpErr::BadTypeOperation),
        },
        (PactType::StringLike(l), PactType::StringLike(r)) => match op {
            OpCode::EQ => Ok(l == r),
            _ => Err(InterpErr::BadTypeOperation),
        },
        (l, PactType::List(r)) => match op {
            OpCode::IN => Ok(r.contains(l)),
            _ => Err(InterpErr::BadTypeOperation),
        }
        _ => Err(InterpErr::TypeMismatch),
    }
}

/// Evaluate a conjunction OpCode given an LHS and RHS boolean
fn eval_conjunction(op: OpCode, lhs: bool, rhs: bool) -> Result<bool, InterpErr> {
    if !op.is_conjunction() {
        return Err(InterpErr::UnexpectedOpCode(op.into()));
    }
    Ok(match op {
        OpCode::AND => lhs & rhs,
        OpCode::OR => lhs | rhs,
        OpCode::XOR => lhs ^ rhs,
        _ => panic!("unreachable"),
    })
}

/// The pact interpreter
/// It evaluates `OpCode`s maintaining the state of the current contract execution
/// Uses the rust type system to encode state, see: https://hoverbear.org/2016/10/12/rust-state-machine-pattern/
/// States provide transformations into other valid states and failure cases.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Interpreter<'a> {
    state: State<'a>,
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

    pub fn interpret(&mut self, op: OpCode) -> Result<(), InterpErr> {
        match &self.state {
            State::Initial => {
                // Only a comparator is valid as the first opcode
                if !op.is_comparator() {
                    return Err(InterpErr::UnexpectedOpCode(op.into()));
                }
                self.state = State::ComparatorQueued {
                    comparator: op,
                    last_assertion_and_conjunction: None,
                };
                Ok(())
            }
            State::ComparatorQueued {
                comparator,
                last_assertion_and_conjunction,
            } => {
                if !op.is_load() {
                    return Err(InterpErr::UnexpectedOpCode(op.into()));
                }

                let lhs = match op {
                    OpCode::LD_INPUT(index) => self
                        .input_data
                        .get(index as usize)
                        .ok_or(InterpErr::MissingIndex(index)),
                    OpCode::LD_USER(index) => self
                        .user_data
                        .get(index as usize)
                        .ok_or(InterpErr::MissingIndex(index)),
                    _ => panic!("unreachable"),
                }?;

                self.state = State::ComparatorLHSLoaded {
                    comparator: *comparator,
                    // TODO: Avoid this alloc
                    lhs: lhs.clone(),
                    last_assertion_and_conjunction: *last_assertion_and_conjunction,
                };
                Ok(())
            }
            State::ComparatorLHSLoaded {
                comparator,
                lhs,
                last_assertion_and_conjunction,
            } => {
                if !op.is_load() {
                    return Err(InterpErr::UnexpectedOpCode(op.into()));
                }
                // We have both sides of the comparator and need to evaluate
                let rhs = match op {
                    OpCode::LD_INPUT(index) => self
                        .input_data
                        .get(index as usize)
                        .ok_or(InterpErr::MissingIndex(index)),
                    OpCode::LD_USER(index) => self
                        .user_data
                        .get(index as usize)
                        .ok_or(InterpErr::MissingIndex(index)),
                    _ => panic!("unreachable"),
                }?;
                let mut result = eval_comparator(*comparator, &lhs, rhs)?;

                // A conjunction is also pending, apply it, merging the last and current result.
                if let Some((last_assertion, conjunction)) = last_assertion_and_conjunction {
                    result = eval_conjunction(*conjunction, *last_assertion, result)?;
                }

                // The assertions and operations upto this point have all been collapsed into
                // a single boolean.
                if result {
                    self.state = State::AssertionTrue;
                } else {
                    self.state = State::AssertionFalse;
                };
                Ok(())
            }
            State::AssertionTrue => {
                if op.is_load() {
                    return Err(InterpErr::UnexpectedOpCode(op.into()));
                }
                if op.is_conjunction() {
                    self.state = State::Conjunctive {
                        last_assertion: true,
                        conjunction: op,
                    };
                } else {
                    self.state = State::ComparatorQueued {
                        comparator: op,
                        last_assertion_and_conjunction: None,
                    };
                };
                Ok(())
            }
            State::AssertionFalse => {
                // There is no continuation of the last assertion.
                // This is now considered a failed clause, and hence the contract has failed
                if op.is_comparator() {
                    self.state = State::Failed;
                    return Ok(());
                }
                // Load is invalid here
                if op.is_load() {
                    return Err(InterpErr::UnexpectedOpCode(op.into()));
                }
                self.state = State::Conjunctive {
                    last_assertion: false,
                    conjunction: op,
                };
                Ok(())
            }
            State::Conjunctive {
                conjunction,
                last_assertion,
            } => {
                // A Conjunction should be followed by a comparator
                if !op.is_comparator() {
                    return Err(InterpErr::UnexpectedOpCode(op.into()));
                }

                self.state = State::ComparatorQueued {
                    comparator: op,
                    last_assertion_and_conjunction: Some((*last_assertion, *conjunction)),
                };
                Ok(())
            }
            State::Failed => Err(InterpErr::Refused),
        }
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
pub enum State<'a> {
    /// The initial interpreter state
    Initial,
    /// The last assertion evaluated as false
    AssertionFalse,
    /// The last assertion evaluated as true
    AssertionTrue,
    /// A comparator operation has been queued
    ComparatorQueued {
        /// The pending comparison
        comparator: OpCode,
        /// This state may have been reached from a conjunctive state
        /// so we preserve this.
        last_assertion_and_conjunction: Option<(bool, OpCode)>,
    },
    /// The LHS of a comparator has been loaded
    ComparatorLHSLoaded {
        /// The pending comparison
        comparator: OpCode,
        /// LHS of a comparator
        lhs: PactType<'a>,
        last_assertion_and_conjunction: Option<(bool, OpCode)>,
    },
    /// The last assertion was followed by a conjunction.
    /// The interpreter is awaiting the next OpCode as the RHS.
    Conjunctive {
        // The last assertion truthiness (LHS of conjunction)
        last_assertion: bool,
        // The conjunction to apply. <LHS> <conjunction> <RHS>
        conjunction: OpCode,
    },
    /// The contract invariants were not maintained
    /// it has failed.
    Failed,
}
