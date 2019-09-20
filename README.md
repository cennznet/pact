# Pact
[![CircleCI](https://circleci.com/gh/cennznet/pact.svg?style=svg)](https://circleci.com/gh/cennznet/pact)  
An embedded contract DSL and toolchain for doughnuts in the CENNZnet permission domain.  


Pact contracts are written in a simple bytecode and execute against dynamic input data to ensure their invariants are upheld.  
It is designed for integration with the CENNZnet blockchain runtime to enable safe, powerful delegated transacitons.

It additionally supports a high-level english like language and compiler. This allows writing human readable "pacts" which the toolchain can interpret; achieving the notion of [Ricardian](https://en.wikipedia.org/wiki/Ricardian_contract).  

![alt text](https://github.com/cennznet/pact/blob/master/design/pact-overview.png)

