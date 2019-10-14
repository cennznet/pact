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

#![cfg_attr(not(feature = "std"), no_std)]

// 'std' is required for parser and compilation
// interpreter can execute in `no_std` environment
#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate pest;

#[cfg(feature = "std")]
#[macro_use]
extern crate pest_derive;

pub mod interpreter;
pub use interpreter::types;

#[cfg(feature = "std")]
pub mod compiler;

#[cfg(feature = "std")]
pub mod parser;
