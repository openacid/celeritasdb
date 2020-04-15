// TODO rename this file, choose a better bin name

use clap::{App, Arg};

use std::sync::Arc;

use tokio;

use epaxos::conf::ClusterInfo;
use epaxos::snapshot::MemEngine;

use cele::Server;

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

    let sto = MemEngine::new().unwrap();

    let cluster = ClusterInfo::from_file(conffn).unwrap();
    let server = Server::new(Arc::new(sto), cluster, node_id.into());

    start(server);
    println!("serve returned");
}

#[tokio::main]
async fn start(mut server: Server) {
    server.start();
    server.join().await.unwrap();
}
