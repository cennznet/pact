# Pact Binary Format v0 (codec)
The pact binary format is 1 version byte, followed by static data section and trailling pact opcodes (bytecode).
`version | datatable | bytecode` or formally,
```
version:   1 LE byte
datatable: DataTable (see datatable codec)
bytecode:  remaining LE bytes
```

# PactType Codec
Codec spec for `PactType` structs

```
type index: 1 LE byte
    0 = StringLike
    1 = Numeric
    2 = List
length: 1 LE byte
data: <length> LE bytes
```

`List` structs are slightly more complex where the internals are another pact type.

For example, a List of three strings would be encoded to:

```
type index: 2 = List
length: 17
data:
    type index: 0 = StringLike
    length: 4
    data: 'know'
    type index: 0 = StringLike
    length: 3
    data: 'the'
    type index: 0 = StringLike
    length: 4
    data: 'game'
```

# DataTable codec
Codec spec for the pact binary datatable.
A DataTable is simply a list of `PactType`s and length prefix.
Its encoded form is a concatenation of encoded `PactType`s
Therefore the process of decoding a `DataTable` from an input buffer is as follows:
1) Read the length byte
2) Decode _l_ PactType's from the input, return _bytes read_, move the offset into the input buffer _bytes read_
2) Repeat until end of input (success) or failure

Encoding is simply:
1) push the length byte _l_
2) push _l_ encoded `PactType`s to the buffer
