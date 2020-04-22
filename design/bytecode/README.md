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
ASSERTION: COMPARATOR LOAD_INDEX LOAD_INDEX | ASSERTION CONJUNCTION ASSERTION
CONJUNCTION: (INVERSION + CONJUNCTION_OP)
COMPARATOR: (INVERSION + LOAD + COMPARATOR_OP)
INVERSION: NORMAL | NOT
CONJUNCTION_OP: AND | OR | XOR
LOAD: LOAD_INPUT_VS_USER | LD_INPUT_VS_INPUT
COMPARATOR_OP: EQ | GT | GTE | IN
```

Goals:
- Embed within doughnuts (compact)
- Executable by the pact interpreter

## OpCodes
8-bit big endian opcode:

| bits    | 7             | 6   |      5 - 2 |    1 - 0 |
|:--------|:-------------:|:---:|:----------:|:--------:|
| purpose | operator type | not |  operation | RESERVED |

  - `bit(7)` determines whether the opcode is a conjunction or a comparator
    ```rust
      // OpCode represents a comparator
      COMP = 0
      // OpCode represents a conjunction
      CONJ = 1
    ```
  - `bit(6)` determines whether a NOT is applied to the conjunction/comparator
    ```rust
      // No logical inversion
      NORMAL = 0
      // Invert the logic of the output
      NOT = 1
    ```
  - `bits(5..2)` (4 bits):
    - for comparators (`bit(7) == 0`):
      | bits    |    5 |      4 - 2 |
      |:--------|-----:|-----------:|
      | purpose | load | comparator |
      - `bit(5)` determines the LOADs to compare:
        ```rust
        // Compare from input to datatable entries
        LOAD_INPUT_VS_USER = 0;
        // Compare from input to input entries
        LOAD_INPUT_VS_INPUT = 1;
        ```
      - `bits(4..2)` determines the comparator operation
        ```rust
        // i == j
        EQ = 0
        // i > j
        GT = 1
        // i >= j
        GTE = 2
        // Whether data[i] is included in the set at data[j]
        IN = 3
        ```
        *Note: `LT` and `LTE` are achieved by using `bit(6)`, the `NOT` operator.*
    - for conjunctions (`bit(7) == 1`):
      ```rust
      // Compute a logical and between A and the next comparator OpCode
      AND = 0
      // Compute an inclusive or between A and the next comparator OpCode
      OR = 1
      // Compute an exclusive or between A and the next comparator OpCode
      XOR = 2
      ```
  - `bits(1..0)` are RESERVED

## Index Codes

A Pact may have up to 16 input arguments and up to 16 entries in a user data table.

This limitation allows comparator indices to be encoded in a single byte:

| bits    |      7 - 4 |     3 - 0 |
|:--------|:----------:|:---------:|
| purpose |  LHS index | RHS index |

## Example Syntax

A series of independent clauses ("implicit and")
```pact
(COMP + LOAD_INPUT_VS_USER + EQ), ((1 << 4) + 0)            # INPUT(1) == USER(0)    | 0x00, 0x10
(COMP + LOAD_INPUT_VS_USER + GT), ((3 << 4) + 1)            # INPUT(3) >  USER(1)    | 0x04, 0x31
(COMP + NOT + LOAD_INPUT_VS_USER + GTE), ((2 << 4) + 3)     # INPUT(2) <  USER(3)    | 0x48, 0x23
```
*Values in brackets represent a single byte (3-bytes per independent clause)*

A multi-assertion clause followed by a single clause (one assertion)
```pact
(COMP + LOAD_INPUT_VS_USER + GT), ((0 << 4) + 1)            # INPUT(0) >  USER(1)    | 0x04, 0x01
(CONJ + NOT + AND)                                         #  NAND                  | 0xC0
(COMP + LOAD_INPUT_VS_USER + EQ), ((0 << 4) + 2)            # INPUT(0) == USER(2)    | 0x00, 0x02
(CONJ + OR)                                                #  OR                    | 0x84
(COMP + NOT + LOAD_INPUT_VS_INPUT + GTE), ((1 << 4) + 3)    # INPUT(1) <  INPUT(3)   | 0x68, 0x13
# A single clause
(COMP + LOAD_INPUT_VS_USER + GTE), ((0 << 4) + 3)           # INPUT(0) >= USER(3)    | 0x08, 0x03
```
