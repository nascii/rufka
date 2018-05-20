use std::thread;
use std::net::{TcpListener, TcpStream};

use crossbeam_channel::Receiver;

use manager::{self, ClientToken};
use decoder::Decoder;
use encoder::Encoder;
use protocol::{IncomingMessage, OutcomingMessage};
use errors::{Error, ErrorKind};

const PORT: u16 = 3001;

fn reader(stream: TcpStream, token: ClientToken) {
    println!("Connected: {:?}", token);

    for message in Decoder::new(stream) {
        println!("< {:?}", message);

        match message {
            IncomingMessage::Create { topic_name } => {
                manager::create(&token, topic_name);
            },
            IncomingMessage::Subscribe { topic_name } => {
                manager::subscribe(&token, &topic_name);
            },
            IncomingMessage::Publish { topic_name, payload } => {
                manager::publish(&token, &topic_name, payload);
            },
            IncomingMessage::Invalid => {
                let error = Error::from(ErrorKind::InvalidCommand);
                manager::bypass(&token, OutcomingMessage::Err(error));
            },
        };
    }

    println!("Disconnected: {:?}", token);

    manager::disconnect(token);
}

fn writer(stream: TcpStream, receiver: Receiver<OutcomingMessage>) {
    let mut encoder = Encoder::new(stream);

    for message in receiver {
        println!("> {:?}", message);
        encoder.write(message);
    }
}

pub fn run() {
    let socket = TcpListener::bind(("127.0.0.1", PORT)).unwrap();

    for stream in socket.incoming() {
        let (token, receiver) = manager::connect();

        let stream = stream.unwrap();
        let writable = stream.try_clone().unwrap();

        thread::spawn(move || reader(stream, token));
        thread::spawn(move || writer(writable, receiver));
    }
}
