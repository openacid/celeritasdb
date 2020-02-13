// TODO rename this file, choose a better bin name

use clap::{App, Arg};

use net2;
use redis;

use net2::TcpBuilder;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::str::from_utf8;

use std::sync::mpsc::{channel, Receiver};
use std::thread;

use parse::Response;

enum Stream {
    Tcp(TcpStream),
}

impl Read for Stream {
    /// Pull some bytes from this source into the specified buffer,
    /// returning how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Stream::Tcp(ref mut s) => s.read(buf),
        }
    }
}

impl Stream {
    /// Creates a new independently owned handle to the underlying socket.
    fn try_clone(&self) -> io::Result<Stream> {
        match *self {
            Stream::Tcp(ref s) => Ok(Stream::Tcp(s.try_clone()?)),
        }
    }

    /// Write a buffer into this object, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            Stream::Tcp(ref mut s) => s.write(buf),
        }
    }
}

pub struct Server {
    /// A list of threads listening for incoming connections
    listen_port: u16,
    listener_threads: Vec<thread::JoinHandle<()>>,
}

impl Server {
    /// Creates a new server
    pub fn new(port: u16) -> Server {
        return Server {
            listen_port: port,
            listener_threads: Vec::new(),
        };
    }

    pub fn run(&mut self) {
        self.start();
        self.join();
    }

    /// Join the listener threads.
    pub fn join(&mut self) {
        while self.listener_threads.len() > 0 {
            let _ = self.listener_threads.pop().unwrap().join();
        }
    }

    fn reuse_address(&self, builder: &TcpBuilder) -> io::Result<()> {
        builder.reuse_address(true)?;
        Ok(())
    }

    /// Listens to a socket address.
    fn api_listen<T: ToSocketAddrs>(&mut self, t: T, tcp_backlog: i32) -> io::Result<()> {
        for addr in t.to_socket_addrs()? {
            let listener = self.make_listener(addr, tcp_backlog)?;

            let th = thread::spawn(move || Self::api_server_loop(listener));
            self.listener_threads.push(th);
        }
        Ok(())
    }

    fn api_server_loop(listener: TcpListener) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Accepted connection to {:?}", stream);
                    thread::spawn(move || {
                        let mut client = Client::tcp(stream);
                        client.run();
                    });
                }
                Err(e) => {
                    println!("Accepting client connection: {:?}", e);
                }
            }
        }
    }

    fn make_listener(&mut self, addr: SocketAddr, backlog: i32) -> io::Result<TcpListener> {
        let builder = match addr {
            SocketAddr::V4(_) => TcpBuilder::new_v4(),
            SocketAddr::V6(_) => TcpBuilder::new_v6(),
        }?;

        self.reuse_address(&builder)?;
        let listener = builder.bind(addr)?.listen(backlog)?;
        Ok(listener)
    }

    /// Starts threads listening to new connections.
    pub fn start(&mut self) {
        let addresses = vec![("127.0.0.1".to_owned(), self.listen_port)];
        for (host, port) in addresses {
            self.api_listen((&host[..], port), 10).unwrap();
            println!("ready to accept connections on port {}", port);
        }
    }
}

/// A client connection
struct Client {
    /// The socket connection
    stream: Stream,
}

impl Client {
    /// Creates a new TCP socket client
    pub fn tcp(stream: TcpStream) -> Client {
        return Client {
            stream: Stream::Tcp(stream),
        };
    }

    /// Creates a thread that writes into the client stream each response received
    fn create_writer_thread(&self, rx: Receiver<Option<Response>>) {
        let mut stream = self.stream.try_clone().unwrap();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(m) => match m {
                    Some(msg) => {
                        println!("to client:");
                        match stream.write(&*msg.as_bytes()) {
                            Ok(_) => (),
                            Err(e) => println!("Error writing to client: {:?}", e),
                        }
                    }
                    None => break,
                },
                Err(_) => break,
            };
        });
    }

    /// Runs all clients commands. The function loops until the client
    /// disconnects.
    pub fn run(&mut self) {
        #![allow(unused_must_use)]
        let (stream_tx, rx) = channel::<Option<Response>>();
        self.create_writer_thread(rx);

        loop {
            let mut buf = vec![0u8; 1024];
            let len = {
                // TODO extend buf
                // Now it requires a complete command in a single packet.
                match self.stream.read(&mut buf[..]) {
                    Ok(r) => {
                        println!("read buf: r={:}, {:?}", r, buf);
                        r
                    }
                    Err(err) => {
                        println!("Reading from client: {:?}", err);
                        break;
                    }
                }
            };

            // client closed connection
            if len == 0 {
                println!("Client closed connection");
                break;
            }

            let v = redis::parse_redis_value(&buf);
            let v = match v {
                Ok(q) => {
                    println!("q= {:?}", q);
                    q
                }
                Err(err) => {
                    // TODO bad protocol handling
                    println!("parse error: {:}", err);
                    break;
                }
            };

            // TODO implementation entry:
            let r = exec_redis_cmd(v);

            // check out the response
            match r {
                // received a response, send it to the client
                Some(response) => {
                    // send to writer thread, which writes bytes into underlying tcp socket.
                    match stream_tx.send(Some(response)) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("error sending to sender thread: {}", e);
                            break;
                        }
                    };
                }
                None => {
                    println!("internal error");
                    break;
                }
            }
        }
        println!("client exit");
        //TODO close socket
    }
}

fn exec_redis_cmd(v: redis::Value) -> Option<Response> {
    // cmd is a nested array: ["set", "a", "1"] or ["set", ["b", "c"], ...]
    // A "set" or "get" redis command is serialized as non-nested array.
    //
    // Flatten one level:
    // tokens is a vec[Value].
    let tokens = match v {
        redis::Value::Bulk(tokens) => tokens,
        _ => vec![],
    };

    // the first token is instruction, e.g. "set" or "get".
    let tok0 = &tokens[0];

    let t = match tok0 {
        redis::Value::Data(d) => d,
        _ => {
            println!("tok0 is not a Data!!!");
            return Some(Response::Error("invalid command".to_owned()));
        }
    };

    println!("instruction: {:?}", t);
    let tok0str = from_utf8(&t).unwrap();

    // execute the command

    match tok0str {
        "FLUSHDB" => Some(Response::Status("OK".to_owned())),
        "SET" => Some(Response::Status("OK".to_owned())),
        "GET" => Some(Response::Integer(42)),
        _ => Some(Response::Error("invalid command".to_owned())),
    }
}

fn main() {
    // TODO standalone version file.
    // TODO add test of command line argument.
    let matches = App::new("cele")
        .version("0.0.1")
        .author("openacid")
        .about("distributed redis")
        .arg(
            Arg::with_name("port")
                .long("port")
                .takes_value(true)
                .help("port to listen"),
        )
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .takes_value(true)
                .help("network address to listen"),
        )
        .get_matches();

    let port_str = matches.value_of("port").unwrap_or("6379");
    let port = port_str.parse::<u16>().unwrap();
    let mut server = Server::new(port);
    println!("Port: {}", port);
    server.run();
}
