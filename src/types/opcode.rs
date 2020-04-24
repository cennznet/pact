use crate::compiler::{LoadSource, SubjectSource};
use crate::interpreter::InterpErr;
use crate::parser::ast;
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

/// Data structure which breaks down the anatomy of an OpCode
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub enum OpCode {
    COMP(Comparator),
    CONJ(Conjunction),
}

/// Comparator OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub struct Comparator {
    pub load: OpLoad,
    pub op: OpComp,
    pub indices: OpIndices,
    pub invert: OpInvert,
}

/// Conjunction OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub struct Conjunction {
    pub op: OpConj,
    pub invert: OpInvert,
}

/// Comparator OpCode Structure
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy)]
pub struct OpIndices {
    pub lhs: u8,
    pub rhs: u8,
}

/// Enum to determine whether a comparator/conjunction
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
    pub fn new(op: OpComp) -> Self {
        Comparator {
            load: OpLoad::INPUT_VS_USER,
            op: op,
            indices: OpIndices { lhs: 0, rhs: 0 },
            invert: OpInvert::NORMAL,
        }
    }

    pub fn load(mut self, load: OpLoad) -> Self {
        self.load = load;
        self
    }

    pub fn indices(mut self, lhs: u8, rhs: u8) -> Self {
        self.indices.lhs = lhs;
        self.indices.rhs = rhs;
        self
    }

    pub fn invert(mut self, invert: OpInvert) -> Self {
        self.invert = invert;
        self
    }

    pub fn loads_from_subjects(mut self, lhs: SubjectSource, rhs: SubjectSource) -> Self {
        // Determine the Load Order
        let (load, flip) = match (lhs.load_source, rhs.load_source) {
            (LoadSource::Input, LoadSource::Input) => (OpLoad::INPUT_VS_INPUT, false),
            (LoadSource::Input, LoadSource::DataTable) => (OpLoad::INPUT_VS_USER, false),
            (LoadSource::DataTable, LoadSource::Input) => (OpLoad::INPUT_VS_USER, true),
            (_, _) => (OpLoad::INPUT_VS_USER, true), // Should not reach here
        };

        // Form the comparator opcode structure and push it out
        self = self.load(load).indices(lhs.index, rhs.index);

        if flip {
            self.flip_indices()
        } else {
            self
        }
    }

    pub fn flip_indices(mut self) -> Self {
        self.indices = OpIndices {
            lhs: self.indices.rhs,
            rhs: self.indices.lhs,
        };
        let (op, invert) = match self.op {
            OpComp::EQ => (self.op, self.invert),
            OpComp::IN => (self.op, self.invert),
            OpComp::GT => (OpComp::GTE, self.invert.invert()),
            OpComp::GTE => (OpComp::GT, self.invert.invert()),
        };
        self.op = op;
        self.invert = invert;
        self
    }

    pub fn apply_imperative(mut self, imperative: &ast::Imperative) -> Self {
        match imperative {
            ast::Imperative::MustBe => {}
            ast::Imperative::MustNotBe => self.invert = self.invert.invert(),
        };
        self
    }
}

impl From<&ast::Comparator> for Comparator {
    fn from(comparator: &ast::Comparator) -> Self {
        match comparator {
            ast::Comparator::Equal => Comparator::new(OpComp::EQ),
            ast::Comparator::GreaterThan => Comparator::new(OpComp::GT),
            ast::Comparator::GreaterThanOrEqual => Comparator::new(OpComp::GTE),
            ast::Comparator::LessThan => Comparator::new(OpComp::GTE).invert(OpInvert::NOT),
            ast::Comparator::LessThanOrEqual => Comparator::new(OpComp::GT).invert(OpInvert::NOT),
            ast::Comparator::OneOf => Comparator::new(OpComp::IN),
        }
    }
}

impl Conjunction {
    pub fn new(op: OpConj) -> Self {
        Conjunction {
            op: op,
            invert: OpInvert::NORMAL,
        }
    }

    pub fn invert(mut self, invert: OpInvert) -> Self {
        self.invert = invert;
        self
    }
}

impl From<&ast::Conjunctive> for Conjunction {
    fn from(conjunctive: &ast::Conjunctive) -> Self {
        match conjunctive {
            ast::Conjunctive::And => Conjunction::new(OpConj::AND),
            ast::Conjunctive::Or => Conjunction::new(OpConj::OR),
        }
    }
}

impl OpInvert {
    pub fn invert(mut self) -> Self {
        self = match self {
            OpInvert::NORMAL => OpInvert::NOT,
            OpInvert::NOT => OpInvert::NORMAL,
        };
        self
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
        match self {
            OpCode::COMP(comp) => {
                let invert_u8: u8 = comp.invert.into();
                let load_u8: u8 = comp.load.into();
                let comp_u8: u8 = comp.op.into();
                invert_u8 | load_u8 | comp_u8
            }
            OpCode::CONJ(conj) => {
                let invert_u8: u8 = conj.invert.into();
                let conj_u8: u8 = conj.op.into();
                OP_TYPE_MASK | invert_u8 | conj_u8
            }
        }
    }
}
