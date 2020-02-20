extern crate protoc_rust;

use protoc_rust::*;

fn main() {
    // TODO test in window, need OS related path
    // TODO use dir-walk to find out all protos
    run(Args {
        out_dir: "../components/epaxos/src/data",
        input: &[
            "../components/epaxos/src/data/protos/command.proto",
            "../components/epaxos/src/data/protos/instance.proto",
            "../components/epaxos/src/data/protos/message.proto",
        ],
        includes: &["../components/epaxos/src/data/protos"],
        customize: Customize {
            ..Default::default()
        },
    })
    .expect("protoc complete");
}
