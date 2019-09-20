// Copyright (C) 2019 Centrality Investments Limited
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
