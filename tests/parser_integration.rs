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

//! Parser integration tests

#![cfg(test)]
use pact::parser;

#[test]
fn it_parses() {
    let ast =
        parser::parse("given parameters $a, $b,  $c 5 must be less than or equal to 123").unwrap();
    println!("{:?}", ast);

    let ast = parser::parse(
        "
      given parameters $hello
      define $world as \"world\"
      $hello must be equal to $world",
    )
    .unwrap();
    println!("{:?}", ast);

    let ast = parser::parse("given parameters $alpha,$whiskey,$foxtrot 5 must be less than or equal to 123 and 5 must not be equal to 6 or 7 must be greater than 12 55555 must not be equal to 123").unwrap();
    println!("{:?}", ast);

    let ast = parser::parse("given parameters $a \"hello world\" must be equal to \"dorem ipsum\" and $a must be less than or equal to 123").unwrap();
    println!("{:?}", ast);

    let ast = parser::parse(
        "
      given parameters $charlie, $tango, $delta
      define $test as 12345
      5 must be less than or equal to 123
      \"hello world\" must be equal to \"dorem ipsum\"",
    )
    .unwrap();
    println!("{:?}", ast);
}

#[test]
fn it_parses_an_integer_list() {
    let _ = parser::parse(
        "given parameters $charlie, $tango, $delta 1337 must be one of [1, 2, 3, 4, 5]",
    )
    .unwrap();

    let _ = parser::parse(
        "
      given parameters $charlie, $tango, $delta
      define $list as [1, 2, 3, 4, 5]
      $delta must be one of $list",
    )
    .unwrap();
}

#[test]
fn it_parses_a_string_list() {
    let _ = parser::parse(
        "given parameters $rick, $astley \"Never\" must be one of [\"Never\", \"gonna\"]",
    )
    .unwrap();

    let _ = parser::parse(
        "
      given parameters $rick, $astley
      define $list as [\"You know\", \"the rules\", \"and so do I\"]
      $rick must be one of $list",
    )
    .unwrap();
}
