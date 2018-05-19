use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver};

use manager;
use bufpool::Buffer;
use decoder::Decoder;

const PORT: u16 = 3001;

fn listener(stream: TcpStream) {
    use protocol::IncomingMessage::*;

    for message in Decoder::new(stream.try_clone().unwrap()) {
        println!("{:?}", message);

        match message {
            Create { topic_name } => {
                manager::create(topic_name);
            },
            Subscribe { topic_name } => {
                let (tx, rx) = channel();

                let writable = stream.try_clone().unwrap();

                manager::subscribe(&topic_name, tx);

                thread::spawn(move || publisher(rx, writable));
            },
            Publish { topic_name, payload } => {
                manager::publish(&topic_name, payload);
            },
        };
    }
}

fn publisher(rx: Receiver<Arc<Buffer>>, mut stream: TcpStream) {
    for buffer in rx {
        stream.write(&buffer).unwrap();
        stream.write(b"\n").unwrap();
    }
}

pub fn run() {
    let socket = TcpListener::bind(("127.0.0.1", PORT)).unwrap();

    for stream in socket.incoming() {
        thread::spawn(move || listener(stream.unwrap()));
    }
}
