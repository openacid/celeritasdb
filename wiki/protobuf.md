# Protobuf related programs

- [ ] TODO grpc: howto
- [ ] TODO comparison with prost(another rust protobuf libl)

`git@github.com:stepancheg/rust-protobuf.git`

这个 git repo. rust protobuf 的东西几乎都在这个 repo 里. 每个目录是一个 workspace
的 member. 这些目录每个发布到 crates.io 上成一个独立 crate.

目录名字有些跟 crate 名字有些不一样.

tikv 里的 protobuf 相关的东西看起来是从这个 repo 中扒过去的, 拆成了独立 git.
回头我再加一下 tikiv 中 git 跟这里组件的对应关系.

目前整理出来的 crate 跟 cmd 相关的依赖/生成关系在这:

```txt

our crate:
protobuf-gen-rust(bin)
  |
  v
crate:
protoc-rust(lib)
> A for rust CLI wrapper.
> 1 It invoke `protoc` to parse `.proto`. --------> crate:
> 2 Then call `protobuf` to receive       -------.  protoc(lib)
>   parsed bytes.                                |  > A general CLI wrapper
> 3 And then call `protobuf-codegen`      -----. |  > for protobuf.
>   to build `.rs`                             | |  > git:stepancheg/rust-protobuf/protoc
> git:stepancheg/rust-protobuf/protoc-rust     | |
                                               | |
                                               | `> crate:
                                               |    protobuf(lib)
                                               |    > Message definition etc.
                                               |    > git:stepancheg/rust-protobuf/protobuf
                                               |
                                               `--> crate:
crate:                                           .> protobuf_codegen(lib)
protobuf-codegen(lib & bin)                      |
> It build a lib:      --------------------------'
> It build a bin:      --------------------------.
> protobuf plugin to generate .rs                |
> git:stepancheg/rust-protobuf/protobuf-codegen  |
                                                 |
                                                 |
cmd:                                             |
protoc   <----------------------.                |
> original `protoc`             |                |
> CLI command                   |                |
                                |                `> cmd:
                                |                .> proto-gen-rust(bin)
                                |                |
Generate .rs from CLI:          |                |
protoc                     \  --'                |
  --plugin=protoc-gen-rust \  -------------------'
  --rust_out=.             \
  hello.proto
```
