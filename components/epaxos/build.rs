extern crate tonic_build;

fn main() {
    // tonic_build::compile_protos("../proto/helloworld.proto").unwrap();

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .type_attribute("OpCode", "#[derive(enum_utils::FromStr)]")
        //TODO command contains vec<u8> that can not be copied.
        // .type_attribute("Command", "#[derive(Copy)]")
        .type_attribute(
            "InstanceId",
            "#[derive(Copy, Eq, Ord, PartialOrd, derive_more::From)]",
        )
        .type_attribute("InstanceIdVec", "#[derive(derive_more::From)]")
        .type_attribute(
            "BallotNum",
            "#[derive(Copy, Eq, Ord, PartialOrd, derive_more::From)]",
        )
        .type_attribute("InvalidRequest", "#[derive(derive_more::From)]")
        .compile(
            &[
                "src/protos/command.proto",
                "src/protos/instance.proto",
                "src/protos/message.proto",
                "src/protos/qpaxos.proto",
                "src/protos/errors.proto",
            ],
            &["src/protos/"],
        )
        .unwrap();

    // https://github.com/hyperium/tonic/blob/master/tonic-build/README.md
}
