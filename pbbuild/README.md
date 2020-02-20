# pbbuild

`pbbuild` is excluded from workspace and its only purpose is to build protobuf
file.

> Compiled `*.rs` from protobuf file are already checked in.
> Unless you modify `components/epaxos/src/data/protos/*.proto`,
> you do not need this step.

Run in this dir:

```
cargo run
```
