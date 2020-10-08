// TODO rename this file, choose a better bin name

use clap::{App, Arg};

use std::sync::Arc;

use tokio;

use cele::init_logger;
use cele::Server;
use epaxos::conf::ClusterInfo;
use storage::{MemEngine, Storage};

#[macro_use]
extern crate slog_global;

fn main() {
    // TODO standalone version file.
    // TODO add test of command line argument.
    let matches = App::new("cele")
        .version("0.0.1")
        .author("openacid")
        .about("distributed redis")
        .arg(
            Arg::with_name("cluster")
                .long("cluster")
                .takes_value(true)
                .help("cluster config in yaml"),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .takes_value(true)
                .help("node id for this server. It must be one key of clusterconf.nodes"),
        )
        .get_matches();

    let conffn = matches.value_of("cluster").unwrap();
    let node_id = matches.value_of("id").unwrap();

    init_logger().unwrap();

    info!("get arg conffn: {:?}", &conffn);
    info!("get arg node_id: {:?}", &node_id);

    let sto = MemEngine::new().unwrap();

    let cluster = ClusterInfo::from_file(conffn).unwrap();
    let server = Server::new(Storage::new(Arc::new(sto)), cluster, node_id.into());

    start(server);
    info!("serve returned");
}

#[tokio::main]
async fn start(mut server: Server) {
    server.start();
    server.join().await.unwrap();
}
