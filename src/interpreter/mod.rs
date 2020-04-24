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
const OP_TYPE_MASK: u8 = 0b0010_0000;
const OP_INVERT_MASK: u8 = 0b0001_0000;
const OP_LOAD_MASK: u8 = 0b0000_1000;
const OP_CONJ_MASK: u8 = 0b0000_1111;
const OP_COMP_MASK: u8 = 0b0000_0111;

pub const INDEX_LHS_MASK: u8 = 0b1111_0000;
pub const INDEX_RHS_MASK: u8 = 0b0000_1111;

pub const INDEX_LHS_SHIFT: usize = 4;
pub const INDEX_RHS_SHIFT: usize = 0;

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

/// Data structure which breaks down the anatomy of an OpCode
/// - implements `Into<u8>`
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub struct OpCode {
    pub op_type: OpType,
    pub invert: OpInvert,
}

/// Comparator OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub struct Comparator {
    pub load: OpLoad,
    pub op: OpComp,
    pub indices: OpIndices,
}

/// Comparator OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub struct OpIndices {
    pub lhs: u8,
    pub rhs: u8,
}

/// Enum to select OpCode Type
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpType {
    /// Load an input var at index into the next free operand register
    COMP(Comparator),
    /// Load a data var at index into the next free operand register
    CONJ(OpConj),
}

/// Enum to determine whether a comparator OpCode
/// should invert the result or not
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpInvert {
    NORMAL,
    NOT,
}

/// Enum to determine whether a comparator OpCode
/// is comparing input to datatable or input to input
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpLoad {
    INPUT_VS_USER,
    INPUT_VS_INPUT,
}

/// Enum of avaliable comparator OpCode operations
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpComp {
    EQ,
    GT,
    GTE,
    IN,
}

/// Enum of avaliable conjunction OpCode operations
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpConj {
    AND,
    OR,
    XOR,
}

impl OpCode {
    pub fn flip_indices(self) -> Self {
        match self.op_type {
            OpType::COMP(comparator) => {
                let indices = OpIndices {
                    lhs: comparator.indices.rhs,
                    rhs: comparator.indices.lhs,
                };
                let (comparator_op, invert) = match comparator.op {
                    OpComp::EQ => (comparator.op, self.invert),
                    OpComp::IN => (comparator.op, self.invert),
                    OpComp::GT => (OpComp::GTE, self.invert.invert()),
                    OpComp::GTE => (OpComp::GT, self.invert.invert()),
                };
                OpCode {
                    op_type: OpType::COMP(Comparator {
                        load: comparator.load,
                        op: comparator_op,
                        indices: indices,
                    }),
                    invert: invert,
                }
            }
            _ => self,
        }
    }

    pub fn get_indices(self) -> OpIndices {
        match self.op_type {
            OpType::COMP(comparator) => comparator.indices,
            _ => OpIndices { lhs: 0, rhs: 0 },
        }
    }

    /// Return the next OpCode by parsing an input byte stream
    pub fn parse(stream: &mut dyn Iterator<Item = &u8>) -> Result<Option<Self>, InterpErr> {
        let op_index = stream.next();
        if op_index.is_none() {
            // This is a valid EOI
            return Ok(None);
        }

        let index = op_index.unwrap();

        // Check if the invert Bit is Set
        let invert = match index & OP_INVERT_MASK {
            0 => OpInvert::NORMAL,
            _ => OpInvert::NOT,
        };

        // Check the Type of OpCode (0 ? comparator : conjunction)
        match index & OP_TYPE_MASK {
            0 => {
                // Comparator
                // Determine load type
                let load = match index & OP_LOAD_MASK {
                    0 => OpLoad::INPUT_VS_USER,
                    _ => OpLoad::INPUT_VS_INPUT,
                };
                // Determine comparator operation
                let op = match index & OP_COMP_MASK {
                    0 => OpComp::EQ,
                    1 => OpComp::GT,
                    2 => OpComp::GTE,
                    3 => OpComp::IN,
                    _ => return Err(InterpErr::InvalidOpCode(*index)),
                };
                // Load indices from the stream
                let indices = if let Some(i) = stream.next() {
                    Ok(*i)
                } else {
                    Err(InterpErr::UnexpectedEOI("expected index"))
                }?;

                // form and return the comparator OpCode
                Ok(Some(OpCode {
                    op_type: OpType::COMP(Comparator {
                        load: load,
                        op: op,
                        indices: OpIndices {
                            lhs: (indices & INDEX_LHS_MASK) >> INDEX_LHS_SHIFT,
                            rhs: (indices & INDEX_RHS_MASK) >> INDEX_RHS_SHIFT,
                        },
                    }),
                    invert: invert,
                }))
            }
            _ => {
                // Conjunction
                let op = match index & OP_CONJ_MASK {
                    0 => OpConj::AND,
                    1 => OpConj::OR,
                    2 => OpConj::XOR,
                    _ => return Err(InterpErr::InvalidOpCode(*index)),
                };
                // form and return the comparator OpCode
                Ok(Some(OpCode {
                    op_type: OpType::CONJ(op),
                    invert: invert,
                }))
            }
        }
    }
}

impl OpInvert {
    fn invert(self) -> Self {
        match self {
            OpInvert::NORMAL => OpInvert::NOT,
            OpInvert::NOT => OpInvert::NORMAL,
        }
    }
}

impl Into<u8> for OpInvert {
    fn into(self) -> u8 {
        match self {
            OpInvert::NORMAL => 0,
            OpInvert::NOT => OP_INVERT_MASK,
        }
    }
}

impl Into<u8> for OpLoad {
    fn into(self) -> u8 {
        match self {
            OpLoad::INPUT_VS_USER => 0,
            OpLoad::INPUT_VS_INPUT => OP_LOAD_MASK,
        }
    }
}

impl Into<u8> for OpComp {
    fn into(self) -> u8 {
        match self {
            OpComp::EQ => 0,
            OpComp::GT => 1,
            OpComp::GTE => 2,
            OpComp::IN => 3,
        }
    }
}

impl Into<u8> for OpConj {
    fn into(self) -> u8 {
        match self {
            OpConj::AND => 0,
            OpConj::OR => 1,
            OpConj::XOR => 2,
        }
    }
}

impl Into<u8> for OpType {
    fn into(self) -> u8 {
        match self {
            OpType::COMP(comp) => {
                let load_u8: u8 = comp.load.into();
                let comp_u8: u8 = comp.op.into();
                load_u8 | comp_u8
            }
            OpType::CONJ(conj) => {
                let conj_u8: u8 = conj.into();
                OP_TYPE_MASK | conj_u8
            }
        }
    }
}

impl Into<u8> for OpIndices {
    fn into(self) -> u8 {
        (self.lhs << INDEX_LHS_SHIFT) & INDEX_LHS_MASK
            | (self.rhs << INDEX_RHS_SHIFT) & INDEX_RHS_MASK
    }
}

/// Convert an OpCode into its u8 index.
/// It does not encode any following parameters
impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        let invert_u8: u8 = self.invert.into();
        let type_u8: u8 = self.op_type.into();
        invert_u8 | type_u8
    }
}

/// Evaluate a comparator OpCode returning its result
fn eval_comparator(
    op: OpComp,
    invert: OpInvert,
    lhs: &PactType,
    rhs: &PactType,
) -> Result<bool, InterpErr> {
    let value = match (lhs, rhs) {
        (PactType::Numeric(l), PactType::Numeric(r)) => match op {
            OpComp::EQ => Ok(l == r),
            OpComp::GT => Ok(l > r),
            OpComp::GTE => Ok(l >= r),
            _ => Err(InterpErr::BadTypeOperation),
        },
        (PactType::StringLike(l), PactType::StringLike(r)) => match op {
            OpComp::EQ => Ok(l == r),
            _ => Err(InterpErr::BadTypeOperation),
        },
        (PactType::List(_), _) => match op {
            _ => Err(InterpErr::BadTypeOperation),
        },
        (l, PactType::List(r)) => match op {
            OpComp::IN => Ok(r.contains(l)),
            _ => Err(InterpErr::BadTypeOperation),
        },
        _ => Err(InterpErr::TypeMismatch),
    }?;

    // Apply inversion if required
    match invert {
        OpInvert::NOT => Ok(!value),
        _ => Ok(value),
    }
}

/// Evaluate a conjunction OpCode given an LHS and RHS boolean
fn eval_conjunction(
    conjunction: &OpConj,
    invert: &OpInvert,
    lhs: &bool,
    rhs: bool,
) -> Result<bool, InterpErr> {
    let value = match conjunction {
        OpConj::AND => lhs & rhs,
        OpConj::OR => lhs | rhs,
        OpConj::XOR => lhs ^ rhs,
    };

    // Apply inversion if required
    Ok(match invert {
        OpInvert::NORMAL => value,
        OpInvert::NOT => !value,
    })
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
        match op.op_type {
            OpType::COMP(comparator) => {
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

                let mut result = eval_comparator(comparator.op, op.invert, &lhs, rhs)?;

                // Evaluate the conjunction if necessary
                match &self.state {
                    State::Conjunctive {
                        last_assertion,
                        conjunction,
                        invert,
                    } => {
                        result = eval_conjunction(conjunction, invert, last_assertion, result)?;
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
            State::AssertionTrue => match op.op_type {
                OpType::COMP(_) => self.execute_comparator(op),
                OpType::CONJ(conjunction) => {
                    self.state = State::Conjunctive {
                        last_assertion: true,
                        conjunction: conjunction,
                        invert: op.invert,
                    };
                    Ok(())
                }
            },
            State::AssertionFalse => {
                match op.op_type {
                    // There is no continuation of the last assertion.
                    // This is now considered a failed clause, and hence the contract has failed
                    OpType::COMP(_) => {
                        self.state = State::Failed;
                        Ok(())
                    }
                    // The conjunction will determine whether the contract has failed or succeeded
                    OpType::CONJ(conjunction) => {
                        self.state = State::Conjunctive {
                            last_assertion: false,
                            conjunction: conjunction,
                            invert: op.invert,
                        };
                        Ok(())
                    }
                }
            }
            State::Conjunctive {
                last_assertion: _,
                conjunction: _,
                invert: _,
            } => {
                // A Conjunction must be followed by a comparator
                match op.op_type {
                    OpType::COMP(_) => self.execute_comparator(op),
                    OpType::CONJ(_) => {
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
        conjunction: OpConj,
        invert: OpInvert,
    },
    /// The contract invariants were not maintained
    /// it has failed.
    Failed,
}
