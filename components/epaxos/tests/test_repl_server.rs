use tokio;
use tonic;
use tonic::transport::Server;

use tokio::time::delay_for;

use std::time::Duration;

use epaxos::qpaxos as qp;
use epaxos::MyQPaxos;

#[test]
fn test_repl_server() {
    _repl_server();
}

#[tokio::main]
async fn _repl_server() {
    let addr = "127.0.0.1:4444".parse().unwrap();

    // This channel is for shutting down the server
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // start a replication server in a coroutine

    let qp = MyQPaxos::default();
    let s = Server::builder().add_service(qp::QPaxosServer::new(qp));

    tokio::spawn(async move {
        println!("spawned");
        s.serve_with_shutdown(addr, async {
            rx.await.ok();
        })
        .await
        .unwrap();
    });

    println!("serving");

    // Wait for server to setup.
    // TODO replace this with loop of trying connecting.
    delay_for(Duration::from_millis(1_000)).await;

    let mut client = qp::QPaxosClient::connect("http://127.0.0.1:4444")
        .await
        .unwrap();

    let inst = qp::Instance {
        ..Default::default()
    };

    // Document said the request should be wrapped by a tonic::Request.
    // Do not know why. It seems to work fine with a protobuf message.

    // let request = Request::new(message::Request::accept());
    // let request = message::Request::accept().into();
    let request = qp::MakeRequest::accept(0, &inst);

    let response = client.accept(request).await.unwrap();

    println!("RESPONSE={:?}", response);

    // shut up or shut down?:)
    let _ = tx.send(());
}
