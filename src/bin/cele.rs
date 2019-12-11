// TODO rename this file, choose a better bin name

use net2;

use net2::TcpBuilder;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use std::sync::mpsc::{channel, Receiver};
use std::thread;

use parse::*;

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
    listener_threads: Vec<thread::JoinHandle<()>>,
}

impl Server {
    /// Creates a new server
    pub fn new() -> Server {
        return Server {
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
    fn listen<T: ToSocketAddrs>(&mut self, t: T, tcp_backlog: i32) -> io::Result<()> {
        for addr in t.to_socket_addrs()? {
            let builder = match addr {
                SocketAddr::V4(_) => TcpBuilder::new_v4(),
                SocketAddr::V6(_) => TcpBuilder::new_v6(),
            }?;

            self.reuse_address(&builder)?;
            let listener = builder.bind(addr)?.listen(tcp_backlog)?;

            let th = thread::spawn(move || {
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
            });
            self.listener_threads.push(th);
        }
        Ok(())
    }

    /// Starts threads listening to new connections.
    pub fn start(&mut self) {
        let addresses = vec![("127.0.0.1".to_owned(), 6379)];
        for (host, port) in addresses {
            match self.listen((&host[..], port), 10) {
                Ok(_) => {
                    println!(
                        "The server is now ready to accept connections on port {}",
                        port
                    );
                }
                Err(err) => {
                    println!(
                        "Creating Server TCP listening socket {}:{}: {:?}",
                        host, port, err
                    );
                    continue;
                }
            }
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
                    Some(msg) => match stream.write(&*msg.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => println!("Error writing to client: {:?}", e),
                    },
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

        let mut parser = Parser::new();

        loop {
            // FIXME: is_incomplete parses the command a second time
            if parser.is_incomplete() {
                parser.allocate();
                let len = {
                    let pos = parser.written;
                    let buffer = parser.get_mut();

                    // read socket
                    match self.stream.read(&mut buffer[pos..]) {
                        Ok(r) => r,
                        Err(err) => {
                            println!("Reading from client: {:?}", err);
                            break;
                        }
                    }
                };
                parser.written += len;

                // client closed connection
                if len == 0 {
                    println!("Client closed connection");
                    break;
                }
            }

            // try to parse received command
            let parsed_command = match parser.next() {
                Ok(p) => p,
                Err(err) => {
                    match err {
                        // if it's incomplete, keep adding to the buffer
                        ParseError::Incomplete => {
                            continue;
                        }
                        ParseError::BadProtocol(s) => {
                            let _ = stream_tx.send(Some(Response::Error(s)));
                            break;
                        }
                        _ => {
                            println!("Protocol error from client: {:?}", err);
                            break;
                        }
                    }
                }
            };
            println!("parsed command: {:?}", parsed_command);

            let cmd = match parsed_command.get_str(0) {
                Ok(r) => r,
                Err(_) => "xxx",
            };

            // execute the command

            let r = if cmd == "SET" {
                Ok(Response::Status("OK".to_owned()))
            } else if cmd == "GET" {
                Ok(Response::Integer(123))
            } else {
                Ok(Response::Error("unknown command".to_owned()))
            };

            // check out the response
            match r {
                // received a response, send it to the client
                Ok(response) => {
                    // send to writer thread, which writes bytes into underlying tcp socket.
                    match stream_tx.send(Some(response)) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("error sending to sender thread: {}", e);
                            break;
                        }
                    };
                }
                // no response
                Err(err) => {
                    match err {
                        // There is no reply to send, that's ok
                        ResponseError::NoReply => (),
                        // We have to wait until a sender signals us back and then retry
                        // (Repeating the same command is actually wrong because of the timeout)
                        ResponseError::Wait(_) => {
                            println!("unimplemented: Wait()");
                            break;
                        }
                    }
                }
            }
        }
        println!("client exit");
    }
}


fn main() {
    let mut server = Server::new();
    println!("Port: {}", 6379);
    server.run();
}
