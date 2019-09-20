# Pact Byte Code
Pact byte code is a DSL for expressing logical assertions on dynamic input data.  
The assertions can be independent or linked to form more complex clauses ('and, or').  
Pact's goal is to be an embedded DSL for constraining the use of doughnut certificates.  
Attaching pact bytecode to a doughnut can be thought of as attaching contractual "terms of use".  
It is the pact interpreters job to resolve whether the clauses were maintained or not given
some input data.  

## Grammar
The syntax of pact byte code is organized so that interpretation requires zero look-ahead.  
To acheive this, the language's opcodes are split into 3 loose categories: loads, conjunctions, and comparators.  
- Comparators: express a logical comparison between two things  
- Conjunctions: express a logical join between comparators  
- Load: Load data from a register for comparison

```
CONTRACT: CLAUSE*
CLAUSE: ASSERTION*
ASSERTION: COMPARATOR LOAD LOAD | ASSERTION CONJUNCTION ASSERTION
CONJUNCTION: AND | OR | XOR
COMPARATOR: EQ | GT | GTE | LT | LTE | IN
LOAD: LD_INPUT | LD_USER
```

Goals:  
- Embed within doughnuts (compact)
- Executable by the pact interpreter

## OpCodes
8-bit big endian opcode  
  - 6 high bits: index  
  - 2 low bits: reserved  

comparator and conjunction opcodes are stand-alone 8bit.  
load opcodes are followed by an 8bit index of the datum to load from the target table.  

```rust
// Load a datum from the input data table into the next free register
LD_INPUT = 0
// Load a datum from the user data table into the next free register
LD_USER = 1
/* Conjunctions */
// Compute a logical and between A and the next comparator OpCode
AND = 2
// Compute an inclusive or between A and the next comparator OpCode
OR = 3
// Compute an exclusive or between A and the next comparator OpCode
XOR = 4
/* Comparators */
// i == j
EQ = 5
// i > j
GT = 6
// i >= j
GTE = 7
// Whether data[i] is included in the set at data[j]
IN = 8
// i < j
LT = 9
// i <= j
LTE = 10
```

## Example Syntax

A series of independent clauses ("implicit and")
```pact
EQ LD_INPUT 0 LD_USER 1
GT LD_INPUT 2 LD_USER 2
LT LD_INPUT 1 LD_USER 0 
```

A multi-assertion clause followed by a single clause (one assertion)  
```pact
GT LD_INPUT 0 LD_USER 1
AND
EQ LD_INPUT 0 LD_USER 2
OR
LT LD_INPUT 1 LD_USER 3
# A single clause
GTE LD_INPUT 0 LD_USER 3
```
