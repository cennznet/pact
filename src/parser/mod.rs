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

pub mod ast;

use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct PactParser;

/// Attempt to parse the given `source` string as pact code.  
/// Returns an AST on success, otherwise the relevant error
pub fn parse(source: &str) -> Result<Vec<ast::Node>, Error<Rule>> {
    let mut ast: Vec<ast::Node> = Default::default();
    let pairs = PactParser::parse(Rule::contract, source.trim())?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::input_declaration => {
                let node = pair.into_inner();
                ast.push(ast::Node::InputDeclaration(
                    node.fuse().map(|ident| ident.as_str().into()).collect(),
                ))
            }
            Rule::assertion | Rule::definition => {
                let node = build_ast_from_statement(pair);
                println!("{:?}", node);
                ast.push(node);
            }
            Rule::EOI => {}
            _ => {
                panic!("unreachable: '{}'", pair.as_str());
            }
        }
    }

    Ok(ast)
}

fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> ast::Node {
    match pair.as_rule() {
        Rule::assertion => ast::Node::Clause(build_assertion(pair)),
        Rule::definition => {
            let mut definition = pair.into_inner();
            let identifier = definition.next().unwrap().as_str().into();
            let value = build_value(definition.next().unwrap());

            ast::Node::Definition(identifier, value)
        }
        _ => {
            panic!("Invalid syntax. Expected assertion or definition");
        }
    }
}

// Build an `Assertion` node from a pest input pair
fn build_assertion(pair: pest::iterators::Pair<Rule>) -> ast::Assertion {
    let mut assertion_pair = pair.into_inner();

    let _lhs = assertion_pair.next().unwrap();
    let lhs = match _lhs.as_rule() {
        Rule::identifier => ast::Subject::Identifier(_lhs.as_str().into()),
        Rule::value => ast::Subject::Value(build_value(_lhs)),
        _ => panic!("unreachable"),
    };
    println!("lhs: {:?}", lhs);

    let imperative = match assertion_pair.next().unwrap().as_rule() {
        Rule::must_be => ast::Imperative::MustBe,
        Rule::must_not_be => ast::Imperative::MustNotBe,
        _ => panic!("unreachable"),
    };
    println!("imperative: {:?}", imperative);

    let comparator = match assertion_pair.next().unwrap().as_rule() {
        Rule::eq => ast::Comparator::Equal,
        Rule::gt => ast::Comparator::GreaterThan,
        Rule::gte => ast::Comparator::GreaterThanOrEqual,
        Rule::lt => ast::Comparator::LessThan,
        Rule::lte => ast::Comparator::LessThanOrEqual,
        _ => panic!("unreachable"),
    };
    println!("comparator: {:?}", comparator);

    let _rhs = assertion_pair.next().unwrap();
    let rhs = match _rhs.as_rule() {
        Rule::identifier => ast::Subject::Identifier(_rhs.as_str().into()),
        Rule::value => ast::Subject::Value(build_value(_rhs)),
        _ => panic!("unreachable"),
    };
    println!("rhs: {:?}", rhs);

    let conjoined_assertion = if let Some(c) = assertion_pair.next() {
        let conjunctive = match c.as_rule() {
            Rule::or => ast::Conjunctive::Or,
            Rule::and => ast::Conjunctive::And,
            _ => panic!("unreachable"),
        };
        // TODO: recurse in here to build another clause instead of...
        let rhs = build_assertion(assertion_pair.next().unwrap());
        Some((conjunctive, Box::from(rhs)))
    } else {
        None
    };

    ast::Assertion(lhs, imperative, comparator, rhs, conjoined_assertion)
}

/// Build a `value` node from a pest input pair
fn build_value(pair: pest::iterators::Pair<Rule>) -> ast::Value {
    let value = pair.into_inner().next().unwrap();
    match value.as_rule() {
        Rule::string => {
            // TODO: The generated parser + grammar should ignore '"' but it's not
            ast::Value::StringLike(value.as_str().trim_matches('"').into())
        }
        Rule::integer => ast::Value::Numeric(value.as_str().parse().unwrap()),
        _ => panic!("unreachable"),
    }
}
