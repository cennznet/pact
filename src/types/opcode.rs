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
//! Pact OpCodes
//!
use crate::interpreter::InterpErr;
use alloc::vec::Vec;

#[cfg(feature = "compiler")]
use crate::parser::ast;

#[cfg(feature = "compiler")]
use core::convert::From;

// OpCode masks
const OP_TYPE_MASK: u8 = 0b0010_0000;
const OP_INVERT_MASK: u8 = 0b0001_0000;
const OP_LOAD_MASK: u8 = 0b0000_1000;
const OP_CONJ_MASK: u8 = 0b0000_1111;
const OP_COMP_MASK: u8 = 0b0000_0111;

const INDEX_LHS_MASK: u8 = 0b1111_0000;
const INDEX_RHS_MASK: u8 = 0b0000_1111;

const INDEX_LHS_SHIFT: usize = 4;
const INDEX_RHS_SHIFT: usize = 0;

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

/// Data structure which breaks down the anatomy of an OpCode
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum OpCode {
    COMP(Comparator),
    CONJ(Conjunction),
}

/// Comparator OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Comparator {
    pub load: OpLoad,
    pub op: OpComp,
    pub indices: OpIndices,
    pub invert: bool,
}

/// Conjunction OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Conjunction {
    pub op: OpConj,
    pub invert: bool,
}

/// Comparator OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct OpIndices {
    pub lhs: u8,
    pub rhs: u8,
}

/// Enum to determine whether a comparator OpCode
/// is comparing input to datatable or input to input
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum OpLoad {
    INPUT_VS_USER,
    INPUT_VS_INPUT,
}

/// Enum of avaliable comparator OpCode operations
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum OpComp {
    EQ,
    GT,
    GTE,
    IN,
}

/// Enum of avaliable conjunction OpCode operations
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub enum OpConj {
    AND,
    OR,
    XOR,
}

impl OpCode {
    // Compiles the OpCode object into one or more bytes
    pub fn compile(self, stream: &mut Vec<u8>) {
        stream.push(self.into());
        match self {
            OpCode::COMP(comparator) => stream.push(comparator.indices.into()),
            _ => {}
        };
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
        let invert = (index & OP_INVERT_MASK) == OP_INVERT_MASK;

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
                Ok(Some(OpCode::COMP(Comparator {
                    load: load,
                    op: op,
                    indices: OpIndices {
                        lhs: (indices & INDEX_LHS_MASK) >> INDEX_LHS_SHIFT,
                        rhs: (indices & INDEX_RHS_MASK) >> INDEX_RHS_SHIFT,
                    },
                    invert: invert,
                })))
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
                Ok(Some(OpCode::CONJ(Conjunction {
                    op: op,
                    invert: invert,
                })))
            }
        }
    }
}

impl Comparator {
    // Constructor for `Comparator`
    pub fn new(op: OpComp) -> Self {
        Comparator {
            load: OpLoad::INPUT_VS_USER,
            op: op,
            indices: OpIndices { lhs: 0, rhs: 0 },
            invert: false,
        }
    }

    // Update the `load` field
    pub fn load(mut self, load: OpLoad) -> Self {
        self.load = load;
        self
    }

    // Update the `indices` field
    pub fn indices(mut self, lhs: u8, rhs: u8) -> Self {
        self.indices.lhs = lhs;
        self.indices.rhs = rhs;
        self
    }

    // Update the `invert` field
    pub fn invert(mut self) -> Self {
        self.invert = true;
        self
    }

    // Update the `load` field based on a subject set
    // If lhs = `DataTable` and rhs = `Input`, we need to change sides so that
    // lhs = `Input` and rhs = `DataTable` as per the `OpCode` encoding spec
    pub fn loads_from_subjects(mut self, lhs: SubjectSource, rhs: SubjectSource) -> Self {
        // Determine the Load Order
        let (load, flip) = match (lhs.load_source, rhs.load_source) {
            (LoadSource::Input, LoadSource::Input) => (OpLoad::INPUT_VS_INPUT, false),
            (LoadSource::Input, LoadSource::DataTable) => (OpLoad::INPUT_VS_USER, false),
            (LoadSource::DataTable, LoadSource::Input) => (OpLoad::INPUT_VS_USER, true),
            (_, _) => (OpLoad::INPUT_VS_USER, true), // Should not reach here
        };

        // Apply the load and indices
        self = self.load(load).indices(lhs.index, rhs.index);

        // Apply a flip if neccessary
        if flip {
            self.flip_indices()
        } else {
            self
        }
    }

    // Flips the lhs and rhs indices and applies any necessary changes to the `op` and
    // `invert` parameters to keep the expressions consistent
    pub fn flip_indices(mut self) -> Self {
        self.indices = OpIndices {
            lhs: self.indices.rhs,
            rhs: self.indices.lhs,
        };
        let (op, invert) = match self.op {
            OpComp::EQ => (self.op, self.invert),
            OpComp::IN => (self.op, self.invert),
            OpComp::GT => (OpComp::GTE, !self.invert),
            OpComp::GTE => (OpComp::GT, !self.invert),
        };
        self.op = op;
        self.invert = invert;
        self
    }

    // Applies an `ast::Imperative` to the `invert` parameter
    #[cfg(feature = "compiler")]
    pub fn apply_imperative(mut self, imperative: &ast::Imperative) -> Self {
        match imperative {
            ast::Imperative::MustBe => {}
            ast::Imperative::MustNotBe => self.invert = !self.invert,
        };
        self
    }
}

#[cfg(feature = "compiler")]
impl From<&ast::Comparator> for Comparator {
    // Creates a `Comparator` from an `ast::Comparator` type
    fn from(comparator: &ast::Comparator) -> Self {
        match comparator {
            ast::Comparator::Equal => Comparator::new(OpComp::EQ),
            ast::Comparator::GreaterThan => Comparator::new(OpComp::GT),
            ast::Comparator::GreaterThanOrEqual => Comparator::new(OpComp::GTE),
            ast::Comparator::LessThan => Comparator::new(OpComp::GTE).invert(),
            ast::Comparator::LessThanOrEqual => Comparator::new(OpComp::GT).invert(),
            ast::Comparator::OneOf => Comparator::new(OpComp::IN),
        }
    }
}

impl Conjunction {
    // Constructor for `Conjunction`
    pub fn new(op: OpConj) -> Self {
        Conjunction {
            op: op,
            invert: false,
        }
    }

    // Update the `invert` field
    pub fn invert(mut self) -> Self {
        self.invert = true;
        self
    }
}

#[cfg(feature = "compiler")]
impl From<&ast::Conjunctive> for Conjunction {
    // Creates a `Conjunction` from an `ast::Conjunctive` type
    fn from(conjunctive: &ast::Conjunctive) -> Self {
        match conjunctive {
            ast::Conjunctive::And => Conjunction::new(OpConj::AND),
            ast::Conjunctive::Or => Conjunction::new(OpConj::OR),
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

impl Into<u8> for OpIndices {
    fn into(self) -> u8 {
        (self.lhs << INDEX_LHS_SHIFT) & INDEX_LHS_MASK
            | (self.rhs << INDEX_RHS_SHIFT) & INDEX_RHS_MASK
    }
}

/// Convert an OpCode into its u8 bytecode
impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            OpCode::COMP(comp) => {
                let invert_u8: u8 = if comp.invert { OP_INVERT_MASK } else { 0 };
                let load_u8: u8 = comp.load.into();
                let comp_u8: u8 = comp.op.into();
                invert_u8 | load_u8 | comp_u8
            }
            OpCode::CONJ(conj) => {
                let invert_u8: u8 = if conj.invert { OP_INVERT_MASK } else { 0 };
                let conj_u8: u8 = conj.op.into();
                OP_TYPE_MASK | invert_u8 | conj_u8
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compile_comparators_basic() {
        let mut bytes = Vec::<u8>::default();
        OpCode::COMP(Comparator::new(OpComp::EQ)).compile(&mut bytes);
        OpCode::COMP(Comparator::new(OpComp::GT)).compile(&mut bytes);
        OpCode::COMP(Comparator::new(OpComp::GTE)).compile(&mut bytes);
        OpCode::COMP(Comparator::new(OpComp::IN)).compile(&mut bytes);
        assert_eq!(bytes, vec![0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00,]);
    }

    #[test]
    fn compile_conjunctions_basic() {
        let mut bytes = Vec::<u8>::default();
        OpCode::CONJ(Conjunction::new(OpConj::AND)).compile(&mut bytes);
        OpCode::CONJ(Conjunction::new(OpConj::OR)).compile(&mut bytes);
        OpCode::CONJ(Conjunction::new(OpConj::XOR)).compile(&mut bytes);
        assert_eq!(bytes, vec![0x20, 0x21, 0x22,]);
    }

    #[test]
    fn compile_comparator_advanced() {
        let mut bytes = Vec::<u8>::default();
        OpCode::COMP(
            Comparator::new(OpComp::EQ)
                .load(OpLoad::INPUT_VS_INPUT)
                .invert()
                .indices(11, 3),
        )
        .compile(&mut bytes);
        assert_eq!(bytes, vec![0x18, 0xb3]);
    }

    #[test]
    fn compile_conjunction_advanced() {
        let mut bytes = Vec::<u8>::default();
        OpCode::CONJ(Conjunction::new(OpConj::OR).invert()).compile(&mut bytes);
        assert_eq!(bytes, vec![0x31]);
    }

    #[test]
    fn parse_comparator_basic() {
        let mut stream = [0x00_u8, 0x00_u8].iter();
        let op_code = OpCode::parse(&mut stream).unwrap();
        assert_eq!(op_code, Some(OpCode::COMP(Comparator::new(OpComp::EQ))));
    }

    #[test]
    fn parse_comparator_gt() {
        let mut stream = [0x01_u8, 0x00_u8].iter();
        let op_code = OpCode::parse(&mut stream).unwrap();
        assert_eq!(op_code, Some(OpCode::COMP(Comparator::new(OpComp::GT))));
    }

    #[test]
    fn parse_comparator_indicies() {
        let mut stream = [0x00_u8, 0x5c_u8].iter();
        let op_code = OpCode::parse(&mut stream).unwrap();
        assert_eq!(
            op_code,
            Some(OpCode::COMP(Comparator::new(OpComp::EQ).indices(5, 12)))
        );
    }

    #[test]
    fn parse_comparator_advanced() {
        let mut stream = [0x18_u8, 0x27_u8].iter();
        let op_code = OpCode::parse(&mut stream).unwrap();
        assert_eq!(
            op_code,
            Some(OpCode::COMP(
                Comparator::new(OpComp::EQ)
                    .invert()
                    .load(OpLoad::INPUT_VS_INPUT)
                    .indices(2, 7)
            ))
        );
    }

    #[test]
    fn parse_comparator_invalid() {
        let mut stream = [0x07_u8, 0x00_u8].iter();
        assert_eq!(
            OpCode::parse(&mut stream),
            Err(InterpErr::InvalidOpCode(0x07))
        );
    }

    #[test]
    fn parse_comparator_missing_indices() {
        let mut stream = [0x00_u8].iter();
        assert_eq!(
            OpCode::parse(&mut stream),
            Err(InterpErr::UnexpectedEOI("expected index"))
        );
    }

    #[test]
    fn parse_conjunction_basic() {
        let mut stream = [0x20_u8].iter();
        let op_code = OpCode::parse(&mut stream).unwrap();
        assert_eq!(op_code, Some(OpCode::CONJ(Conjunction::new(OpConj::AND))));
    }

    #[test]
    fn parse_conjunction_xor() {
        let mut stream = [0x22_u8].iter();
        let op_code = OpCode::parse(&mut stream).unwrap();
        assert_eq!(op_code, Some(OpCode::CONJ(Conjunction::new(OpConj::XOR))));
    }

    #[test]
    fn parse_conjunction_advanced() {
        let mut stream = [0x31_u8].iter();
        assert_eq!(
            OpCode::parse(&mut stream).unwrap(),
            Some(OpCode::CONJ(Conjunction::new(OpConj::OR).invert()))
        );
    }

    #[test]
    fn parse_conjunction_invalid() {
        let mut stream = [0x2f_u8].iter();
        assert_eq!(
            OpCode::parse(&mut stream),
            Err(InterpErr::InvalidOpCode(0x2f))
        );
    }
}
