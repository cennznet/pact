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
//! The pact AST
//! It represents a contract composed of definitions and clauses
//!

/// AST node types
#[derive(Debug)]
pub enum Node {
    /// The declaration of input variable names for the contract
    InputDeclaration(Vec<Identifier>),

    /// A statement mapping an identifier to a value
    /// (identifier, value) .e.g ("account", "Qm53w689adflkhnknkjhkj")
    Definition(Identifier, Value),

    /// A high-level construct formed by one or more conjoined assertions
    Clause(Assertion),
}

/// A primitive construct which describes a single invariant
/// (identifier, imperative, comparator, subject)
#[derive(Debug)]
pub struct Assertion(
    pub Subject,
    pub Imperative,
    pub Comparator,
    pub Subject,
    pub Option<(Conjunctive, Box<Self>)>,
);

/// `MustBe` implies `Comparator == true` while `MustNotBe` implies `Comparator == false`
#[derive(Debug)]
pub enum Imperative {
    MustBe,
    MustNotBe,
}

/// Represents a logical join of two clauses
#[derive(Debug)]
pub enum Conjunctive {
    Or,
    And,
}

/// A logical operation to assert
#[derive(Debug)]
pub enum Comparator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    OneOf,
}

/// A subject of a comparator (LHS / RHS).
/// It may be a literal value or an identifier
#[derive(Debug)]
pub enum Subject {
    Value(Value),
    Identifier(Identifier),
}

/// A literal value, used in place for a comparator or on the RHS of a definition
#[derive(Clone, Debug)]
pub enum Value {
    StringLike(String),
    Numeric(u64),
    List(Vec<Value>)
}

pub type Identifier = String;
