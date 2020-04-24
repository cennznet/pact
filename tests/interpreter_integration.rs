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

//! Interpreter integration tests

#![cfg(test)]
use pact::{
    interpreter::{self, InterpErr},
    interpreter::{Comparator, Conjunction, OpCode, OpComp, OpConj, OpInvert, OpLoad},
    types::{Numeric, PactType, StringLike},
};

#[test]
fn it_does_an_eq_comparison() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            // INPUT(1) == USER(1)
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );

    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_a_not_eq_comparison() {
    let input = [
        PactType::Numeric(Numeric(123)),
        PactType::Numeric(Numeric(234)),
    ];
    let user = [PactType::Numeric(Numeric(123))];
    let neq = OpCode::COMP(Comparator::new(OpComp::EQ).invert(OpInvert::NOT));

    let result = interpreter::interpret(&input, &user, &[neq.into(), 0x00]);
    assert_eq!(result, Ok(false));

    let result = interpreter::interpret(&input, &user, &[neq.into(), 0x10]);
    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_a_lt_comparison_ok() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(99))],
        &[PactType::Numeric(Numeric(100))],
        &[
            // INPUT(1) < USER(1)
            OpCode::COMP(Comparator::new(OpComp::GTE).invert(OpInvert::NOT)).into(),
            0x00,
        ],
    );

    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_an_lte_comparison_ok() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(100))],
        &[PactType::Numeric(Numeric(100))],
        &[
            // INPUT(1) <= USER(1)
            OpCode::COMP(Comparator::new(OpComp::GT).invert(OpInvert::NOT)).into(),
            0x00,
        ],
    );

    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_a_gt_comparison_ok() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(101))],
        &[PactType::Numeric(Numeric(100))],
        &[
            // INPUT(1) > USER(1)
            OpCode::COMP(Comparator::new(OpComp::GT)).into(),
            0x00,
        ],
    );

    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_a_gte_comparison_ok() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(100))],
        &[PactType::Numeric(Numeric(100))],
        &[
            // INPUT(1) < USER(1)
            OpCode::COMP(Comparator::new(OpComp::GTE)).into(),
            0x00,
        ],
    );

    assert_eq!(result, Ok(true));
}

#[test]
fn input_to_input_works() {
    let eq = OpCode::COMP(Comparator::new(OpComp::EQ).load(OpLoad::INPUT_VS_INPUT));
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::Numeric(Numeric(123)),
        ],
        &[],
        &[eq.into(), 0x01],
    );
    assert_eq!(result, Ok(true));
}

#[test]
fn it_fails_with_bad_type_operation_on_stringlike() {
    let bad_op_codes = vec![
        OpCode::COMP(Comparator::new(OpComp::GTE)).into(),
        OpCode::COMP(Comparator::new(OpComp::GT)).into(),
        OpCode::COMP(Comparator::new(OpComp::GTE).invert(OpInvert::NOT)).into(),
        OpCode::COMP(Comparator::new(OpComp::GT).invert(OpInvert::NOT)).into(),
        OpCode::COMP(Comparator::new(OpComp::IN)).into(),
    ];
    for op in bad_op_codes.into_iter() {
        let result = interpreter::interpret(
            &[PactType::StringLike(StringLike(b"test"))],
            &[PactType::StringLike(StringLike(b"test"))],
            &[op, 0x00],
        );

        assert_eq!(result, Err(InterpErr::BadTypeOperation));
    }
}

#[test]
fn it_fails_with_invalid_op_code() {
    let result = interpreter::interpret(&[], &[], &[63]); // An arbitrary undefined opcode
    assert_eq!(result, Err(InterpErr::InvalidOpCode(63)));
}

#[test]
fn load_input_fails_with_unexpected_end_of_input() {
    let result = interpreter::interpret(
        &[],
        &[],
        &[OpCode::COMP(Comparator::new(OpComp::GTE)).into()],
    );
    assert_eq!(result, Err(InterpErr::UnexpectedEOI("expected index")));
}

#[test]
fn it_fails_when_comparator_is_not_followed_by_load_indexes() {
    let result = interpreter::interpret(
        &[],
        &[],
        &[OpCode::COMP(Comparator::new(OpComp::EQ)).into()],
    );
    assert_eq!(result, Err(InterpErr::UnexpectedEOI("expected index")));
}

#[test]
fn load_fails_with_missing_index() {
    let result = interpreter::interpret(
        &[],
        &[],
        &[OpCode::COMP(Comparator::new(OpComp::EQ)).into(), 0x05],
    );
    assert_eq!(result, Err(InterpErr::MissingIndex(0)));

    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(101))],
        &[PactType::Numeric(Numeric(101))],
        &[OpCode::COMP(Comparator::new(OpComp::EQ)).into(), 0x05],
    );
    assert_eq!(result, Err(InterpErr::MissingIndex(5)));
}

#[test]
fn load_input_to_input_fails_with_missing_index_2() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(123))],
        &[
            PactType::Numeric(Numeric(123)),
            PactType::Numeric(Numeric(123)),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ).load(OpLoad::INPUT_VS_INPUT)).into(),
            0x01,
        ],
    );
    assert_eq!(result, Err(InterpErr::MissingIndex(1)));
}

#[test]
fn it_fails_when_first_op_code_is_not_a_comparator() {
    let result = interpreter::interpret(
        &[],
        &[],
        &[OpCode::CONJ(Conjunction::new(OpConj::AND)).into()],
    );
    assert_eq!(
        result,
        Err(InterpErr::UnexpectedOpCode(
            OpCode::CONJ(Conjunction::new(OpConj::AND)).into()
        ))
    );
}

#[test]
fn it_does_an_and_conjunction_ok() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::AND)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_an_or_conjunction_ok() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(321)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::OR)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(true));

    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::OR)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_a_xor_conjunction_ok() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(321)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::XOR)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_an_and_conjunction_evaluates_false() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(321)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::AND)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_an_or_conjunction_evaluates_false() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(321)),
            PactType::StringLike(StringLike(b"world hello")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::OR)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_a_xor_conjunction_evaluates_false() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::XOR)).into(),
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );
    assert_eq!(result, Ok(false));
}

#[test]
fn it_fails_with_unexpected_end_of_input_no_rhs_of_conjunction() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(123))],
        &[PactType::Numeric(Numeric(123))],
        &[
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            OpCode::CONJ(Conjunction::new(OpConj::AND)).into(),
        ],
    );
    assert_eq!(
        result,
        Err(InterpErr::UnexpectedEOI("incomplete operation"))
    );
}

#[test]
fn it_does_an_eq_comparison_evaluates_false() {
    let result = interpreter::interpret(
        &[
            PactType::Numeric(Numeric(123)),
            PactType::StringLike(StringLike(b"hello world")),
        ],
        &[
            PactType::Numeric(Numeric(321)),
            PactType::StringLike(StringLike(b"world hello")),
        ],
        &[
            // EQ LD_INPUT(0) LD_USER(0)
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x00,
            // EQ LD_INPUT(1) LD_USER(1)
            OpCode::COMP(Comparator::new(OpComp::EQ)).into(),
            0x11,
        ],
    );

    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_an_lt_comparison_evaluates_false() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(100))],
        &[PactType::Numeric(Numeric(99))],
        &[
            OpCode::COMP(Comparator::new(OpComp::GTE).invert(OpInvert::NOT)).into(),
            0x00,
        ],
    );

    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_an_lte_comparison_evaluates_false() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(101))],
        &[PactType::Numeric(Numeric(100))],
        &[
            OpCode::COMP(Comparator::new(OpComp::GT).invert(OpInvert::NOT)).into(),
            0x00,
        ],
    );

    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_a_gt_comparison_evaluates_false() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(100))],
        &[PactType::Numeric(Numeric(101))],
        &[OpCode::COMP(Comparator::new(OpComp::GT)).into(), 0x00],
    );

    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_a_gte_comparison_evaluates_false() {
    let result = interpreter::interpret(
        &[PactType::Numeric(Numeric(100))],
        &[PactType::Numeric(Numeric(101))],
        &[OpCode::COMP(Comparator::new(OpComp::GTE)).into(), 0x00],
    );

    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_a_numeric_in_comparison() {
    let input_data = [PactType::Numeric(Numeric(2)), PactType::Numeric(Numeric(5))];
    let user_data = [PactType::List(vec![
        PactType::Numeric(Numeric(1)),
        PactType::Numeric(Numeric(2)),
    ])];

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x00],
    );
    assert_eq!(result, Ok(true));

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x10],
    );
    assert_eq!(result, Ok(false));
}

#[test]
fn it_does_a_string_in_comparison() {
    let input_data = [
        PactType::StringLike(StringLike(b"Never gonna")),
        PactType::StringLike(StringLike(b"give you up")),
    ];
    let user_data = [PactType::List(vec![
        PactType::StringLike(StringLike(b"Never gonna")),
        PactType::StringLike(StringLike(b"let you down")),
    ])];

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x00],
    );
    assert_eq!(result, Ok(true));

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x10],
    );
    assert_eq!(result, Ok(false));
}

#[test]
fn it_fails_with_lhs_list_for_in_comparison() {
    let input_data = [PactType::List(vec![
        PactType::Numeric(Numeric(1)),
        PactType::Numeric(Numeric(2)),
    ])];
    let user_data = [PactType::Numeric(Numeric(2)), PactType::Numeric(Numeric(5))];

    // List in Numeric
    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x00],
    );
    assert_eq!(result, Err(InterpErr::BadTypeOperation));

    // List in List
    let result = interpreter::interpret(
        &input_data,
        &input_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x00],
    );
    assert_eq!(result, Err(InterpErr::BadTypeOperation));
}

#[test]
fn it_does_an_in_comparison_with_a_mixed_list() {
    let input_data = [PactType::Numeric(Numeric(1931))];
    let user_data = [PactType::List(vec![
        PactType::StringLike(StringLike(b"It's alive! It's alive!")),
        PactType::Numeric(Numeric(1931)),
    ])];

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[OpCode::COMP(Comparator::new(OpComp::IN)).into(), 0x00],
    );
    assert_eq!(result, Ok(true));
}

#[test]
fn it_does_a_not_in_comparison() {
    let input_data = [PactType::Numeric(Numeric(2)), PactType::Numeric(Numeric(5))];
    let user_data = [PactType::List(vec![
        PactType::Numeric(Numeric(1)),
        PactType::Numeric(Numeric(2)),
    ])];

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[
            OpCode::COMP(Comparator::new(OpComp::IN).invert(OpInvert::NOT)).into(),
            0x00,
        ],
    );
    assert_eq!(result, Ok(false));

    let result = interpreter::interpret(
        &input_data,
        &user_data,
        &[
            OpCode::COMP(Comparator::new(OpComp::IN).invert(OpInvert::NOT)).into(),
            0x10,
        ],
    );
    assert_eq!(result, Ok(true));
}

#[test]
fn it_fails_for_invalid_list_operators() {
    let input_data = [PactType::Numeric(Numeric(2))];
    let user_data = [PactType::List(vec![
        PactType::Numeric(Numeric(1)),
        PactType::Numeric(Numeric(2)),
    ])];

    let invalid_code_set = [
        OpCode::COMP(Comparator::new(OpComp::EQ)),
        OpCode::COMP(Comparator::new(OpComp::GT).invert(OpInvert::NOT)),
        OpCode::COMP(Comparator::new(OpComp::GTE).invert(OpInvert::NOT)),
        OpCode::COMP(Comparator::new(OpComp::GT)),
        OpCode::COMP(Comparator::new(OpComp::GTE)),
    ];

    for invalid_code in &invalid_code_set {
        let result = interpreter::interpret(
            &input_data,
            &user_data,
            &[invalid_code.clone().into(), 0x00],
        );
        assert_eq!(result, Err(InterpErr::BadTypeOperation));
    }
}
