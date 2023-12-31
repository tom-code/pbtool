
Utility to encode/decode protobuf data into human readable format (json).


![build](https://github.com/tom-code/pbtool/actions/workflows/rust.yml/badge.svg)


Example use:
```
get help:
pbtool help
pbtool encode --help
pbtool decode --help
```


Encode using protobuf defintion in test/a.proto type M1. Read input data from test/a.json.
Read additional protobuf dependencies from directory test.
Output store into file a.bin

```
pbtool encode test/a.proto .M1 test/a.json  -o a.bin -i test
```


Encode using protobuf defintion in test/a.proto type M1. Read input data from test/a.json.
Read additional protobuf dependencies from directory test.
Output store into file a.hex in hexadecimal format

```
pbtool encode test/a.proto .M1 test/a.json  -o a.hex -i test -f hex
```


Decode using protobuf defintion in test/a.proto type M1. Read input data from file a.bin.
Read additional protobuf dependencies from directory test.


```
pbtool decode test/a.proto .M1 @a.bin -i test
```



Decode using protobuf defintion in test/a.proto type M1. Read input data from file a.hex in hexadecimal format.
Read additional protobuf dependencies from directory test.
Store output json in xx.json.


```
pbtool decode test/a.proto .M1 @a.hex -i test -f hex -o xx.json
```
