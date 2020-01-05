## help information

```
cargo run -- -h
```


## run from crate

### with files specified

```
cargo run -- -o test/dest  -s test/protos  -f "a.proto b.proto"
```

### with all .proto files under `-s`

```
cargo run -- -o test/dest  -s test/protos
```


## example

### layout before

```
└── test
    ├── dest
    └── protos
        ├── items
        │   └── items.proto
        └── upper_items.proto
```

### run

```
cargo run -- -o test/dest  -s test/protos
```

### layout after

```
└── test
    ├── dest
    │   ├── items
    │   │   └── items.rs
    │   └── upper_items.rs
    └── protos
        ├── items
        │   └── items.proto
        └── upper_items.proto
```
