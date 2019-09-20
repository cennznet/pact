# Interpreter
The pact interpreter executes bytecode representing logical clauses in a contract.  
It is able to execute the bytecode without any look ahead, failing when a clause (one or more assertions) is false.  
It will fail-fast on the first failed clause, halting execution.  
The final result of execution is a boolean, showing whether the contract was upheld or not.  

The interpreter maintains a few pieces of information in order to track the state and "truthiness" of an
executing contract.  
- current assertion truthiness (register _A_)  
- pending conjunction          (register _B_)  
- pending comparator           (register _C_)  
- comparator LHS               (register _X_)  
- comparator RHS               (register _Y_)  

A state machine outlining the process of executing pact byte code  
![alt-text](../pact-interpreter-state-machine.png "state machine")]  

## Interpreter
The pact interpreter evaluates pact byte code. It checks a contract for correctness (syntax)
and evaluates its clauses yielding a simple boolean result.  

## PactType Semantics and DataTables
Recall, pact revolves around making simple comparisons between two operands (LHS, RHS).  
Typically the LHS would be an input parameter while the RHS would be something the user defined.  
e.g. A statement like `eq(amount, 100)`.
Encoded as pact byte code this would become: 
```
EQ
LD_INPUT 0
LD_USER 0
```
`LD_INPUT 0` means load the parameter at index 0 of the input table and
`LD_USER 0` means load the parameter at index 0 of the user table.  

Many VMs encode types into opcodes themselves making a distinction between a load or equality comparison
on a string vs. 32-bit number.  
Instead, the pact interpreter requires input and user "data tables" are given as input along with bytecode
for execution.

A data table is an array of pact data types. Load opcodes reference indexes in either of these tables  
marking them for comparison operations. 
Pact data types are either _numeric_ or _string-like_, this difference is enough for the interpreter to semantically
validate the type of comparison that is supported on a type.  
A string-like type does not support `<, <=, >, >=` style comparisons, while a numeric type would not support a
"looks-like" or fuzzy match comparator, were one to exist.  
Additionally, the interpreter can check that the LHS and RHS have matching datatypes or void the comparison.  
