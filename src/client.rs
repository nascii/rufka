use std::io::{BufReader, ErrorKind as IoErrorKind, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::time::Duration;

use errors::{Error, ErrorKind, Result};
use protocol::{self, Container, Exchange, Request, Response, Transaction};

fn send(mut writer: impl Write, exchange: Exchange) -> Result<()> {
    let transaction = Transaction {
        correlation: 42,
        exchange,
    };

    let config = protocol::config();

    let container = Container {
        size: config.serialized_size(&transaction)? as i32,
        transaction,
    };

    let buffer = config.serialize(&container)?;

    writer.write_all(&buffer)?;

    Ok(())
}

fn recv(reader: impl Read) -> Result<Exchange> {
    let reader = BufReader::new(reader);
    let config = protocol::config();

    let container: Container = config.deserialize_from(reader)?;

    Ok(container.transaction.exchange)
}

pub fn status() {
    let addr = "127.0.0.1:3001".parse().unwrap();

    let stream = match TcpStream::connect_timeout(&addr, Duration::new(3, 0)) {
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
    };

    match send(&stream, Exchange::Request(Request::Ping)) {
        Ok(_) => {}
        Err(Error(ErrorKind::Io(err), _)) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
        Err(err) => panic!("Unexpected error: {}", err),
    }

    match recv(stream) {
        Ok(Exchange::Response(Response::Ok)) => println!("Ok"),
        Err(Error(ErrorKind::Io(err), _)) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
        _ => {
            eprintln!("Invalid response");
            exit(1);
        }
    }
}
