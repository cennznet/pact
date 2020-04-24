use crate::interpreter::InterpErr;

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
    pub fn flip_indices(self) -> Self {
        let indices = OpIndices {
            lhs: self.indices.rhs,
            rhs: self.indices.lhs,
        };
        let (op, invert) = match self.op {
            OpComp::EQ => (self.op, self.invert),
            OpComp::IN => (self.op, self.invert),
            OpComp::GT => (OpComp::GTE, self.invert.invert()),
            OpComp::GTE => (OpComp::GT, self.invert.invert()),
        };
        Comparator {
            load: self.load,
            op: op,
            indices: indices,
            invert: invert,
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
