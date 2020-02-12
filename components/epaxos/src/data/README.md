## Introduction

all proto files and generated files are placed here.

and ** DO NOT USE THIS MODULE DIRECTLY **


## Usage

### Prepare

create `protobuf-gen-rust` under `celeritasdb/components/protobuf-gen-rust`.

Place the executable binary named protobuf-gen-rust under the directory in your
`PATH`.


### generate all protos once

go to path `celeritasdb/components/epaxos/src/data`

```
protobuf-gen-rust -o ./ -s protos
```

the generated rs files would be placed under `celeritasdb/components/epaxos/src/data`.


### generate specified protos only

go to path `celeritasdb/components/epaxos/src/data`

```
protobuf-gen-rust -o ./ -s protos -f command.proto
```

the result is the same as above.

### update mod.rs under `celeritasdb/components/epaxos/src/data`

re-export your new module in mod.rs.
