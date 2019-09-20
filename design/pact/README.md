# Pact
A DSL for describing contractual agreements in a doughnut permission certificate.  
Goals:  
- Near english syntax with zero ambiguity  
- Compile to terse byte code

## Example Syntax
```pact
given parameters $payee, $amount, $asset_id

define $assets as [16001, 16010]

$payee must be equal to "alice" and $amount must not be greater than 100
$asset_id must be in $assets
```

## Grammar
```
contract:     header statement*
header:       GIVEN VARIABLES: ident_list
statement:    assertion | definition
assertion:    ident imperative comparator+ value | assertion conjunction assertion | assertion conjunction assertion
definition:   WHERE ident IS DEFINED AS value
imperative:   MUST BE | MUST NOT BE
comparator:   LESS THAN | GREATER THAN | GREATER THAN OR EQUAL TO | LESS THAN OR EQUAL TO | IN
conjunction:  OR | AND | BUT NOT BOTH
value:        string | integer | ident
string:       "[a-Z0-9]+"
integer:      [0-9]+
ident:        $([a-Z]+[0-9]*)*
ident_list:   ident | ident_list, ident
```
## Tables
The input table is an ordered array of values. Order corresponds to the call input parameter ordering  
e.g. `generic-asset.transfer(destination, amount, asset_id) -> [destination, amount, asset_id]`  
This is provided by the virtual machine at runtime.  

The data table is an array of user defined static data.  
Order corresponds to the order of use / declaration.  

data: ["hello", "world", 1, 2, 3, "aloha"]
