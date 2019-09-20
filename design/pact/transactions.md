# Pact for Transaction Constraints
Within CENNZnet a doughnut can allow a transaction to be made with delegated authority.  
This is accomplished by setting the "module" and "method" values in the doughnut.  
However, parameterizing arguments to methods becomes tricky and this is where pact becomes useful.  
Consider a transaction made for a generic asset transfer, it has the signature:
 `generic_asset.transfer(asset_id, amount, who)`
`asset_id` and `amount` are intergers while `who` is a public key.  

A pact contract could be written to limit these parameters to certain values, for instance:
```pact
given $asset_id, $amount, $who
$asset_id must be 16001
$amount must be less than 100
$payee must be "charlie"
```

CENNZnet will invoke the pact interpreter with the contract and transaction values to allow or disallow the transaction accordingly.  
