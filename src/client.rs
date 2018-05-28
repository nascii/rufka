use std::io::{self, BufReader, ErrorKind as IoErrorKind, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::time::Duration;

use protocol::{self, Container, Exchange, Request, Response, Transaction};

fn connect() -> TcpStream {
    let addr = "127.0.0.1:3001".parse().unwrap();
    let timeout = Duration::new(3, 0);

    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(stream) => stream,
        Err(ref err) if err.kind() == IoErrorKind::ConnectionRefused => {
            eprintln!("Server is not running");
            exit(1);
        }
        Err(ref err) if err.kind() == IoErrorKind::TimedOut => {
            eprintln!("Timed out");
            exit(1);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    }
}

fn send(mut writer: impl Write, exchange: Exchange) {
    let transaction = Transaction {
        correlation: 42,
        exchange,
    };

    let config = protocol::config();

    let container = Container {
        size: config.serialized_size(&transaction).unwrap() as i32,
        transaction,
    };

    let buffer = config.serialize(&container).unwrap();

    if let Err(err) = writer.write_all(&buffer) {
        eprintln!("Error: {}", err);
        exit(1);
    }
}

fn recv(reader: impl Read) -> Exchange {
    let reader = BufReader::new(reader);
    let config = protocol::config();

    match config.deserialize_from::<_, Container>(reader) {
        Ok(container) => container.transaction.exchange,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    }
}

pub fn status() {
    let stream = connect();

    send(&stream, Exchange::Request(Request::Ping));

    match recv(&stream) {
        Exchange::Response(Response::Ok) => println!("Ok"),
        exchange => {
            eprintln!("Invalid exchange: {:#?}", exchange);
            exit(1);
        }
    }
}

pub fn create(topic: String) {
    let stream = connect();

    let request = Request::Create {
        topic: topic.into(),
    };

    send(&stream, Exchange::Request(request));

    match recv(&stream) {
        Exchange::Response(Response::Ok) => println!("Ok"),
        exchange => {
            eprintln!("Invalid exchange: {:#?}", exchange);
            exit(1);
        }
    }
}

pub fn publish(topic: String, key: String) {
    let stream = connect();

    let mut value = Vec::new();

    io::stdin().read_to_end(&mut value).unwrap();

    let request = Request::Publish {
        topic: topic.into(),
        key: key.into(),
        value: value.into(),
    };

    send(&stream, Exchange::Request(request));

    match recv(&stream) {
        Exchange::Response(Response::Ok) => println!("Ok"),
        Exchange::Response(Response::UnknownTopic) => println!("Unknown topic"),
        exchange => {
            eprintln!("Invalid exchange: {:#?}", exchange);
            exit(1);
        }
    }
}
