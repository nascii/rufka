use std::sync::Arc;

use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use codec::Codec;
use errors::Error;
use peer::Peer;
use protocol::{Exchange, Message, Request, Response, Transaction};
use state::State;
use topic::Topic;

fn process(stream: TcpStream, state: Arc<State>) {
    let addr = stream.peer_addr().unwrap();

    println!("{}: connected", addr);

    let (writer, reader) = stream.framed(Codec::new()).split();

    let (peer, rx) = Peer::new(addr);

    let peer = Arc::new(peer);

    let receiving = receiver(state.clone(), reader, peer.clone());
    let sending = sender(rx, writer, peer.clone());

    // TODO: wait last transaction before closing.

    let connection = receiving.select(sending).then(move |_| {
        let mut topics = state.topics.write();

        for topic in peer.unsubscribe_all() {
            if let Some(topic) = topics.get_mut(&topic) {
                topic.unsubscribe(&peer);
            }
        }

        println!("{}: disconnected", addr);
        Ok(())
    });

    tokio::spawn(connection);
}

// TODO: stream buffering.

fn receiver(
    state: Arc<State>,
    reader: impl Stream<Item = Transaction, Error = Error>,
    peer: Arc<Peer>,
) -> impl Future<Item = (), Error = ()> {
    let peer_clone = peer.clone();

    let reader = reader.map_err(move |_| {
        let transaction = Transaction {
            correlation: 0,
            exchange: Exchange::Response(Response::InvalidCommand),
        };

        println!("> {}: {:?}", peer_clone.addr, transaction);

        peer_clone.send(transaction);
        ()
    });

    reader.for_each(move |transaction| {
        println!("> {}: {:?}", peer.addr, transaction);

        let correlation = transaction.correlation;

        let request = if let Exchange::Request(request) = transaction.exchange {
            request
        } else {
            // TODO: logging.
            return Ok(());
        };

        match request {
            Request::Publish { topic, key, value } => {
                let mut topics = state.topics.read();

                let response = if let Some(topic) = topics.get(&topic) {
                    topic.send(&Message {
                        offset: 0,
                        key,
                        value,
                    });

                    Response::Ok
                } else {
                    Response::UnknownTopic
                };

                peer.send(Transaction {
                    correlation,
                    exchange: Exchange::Response(response),
                });
            }
            Request::Create { topic } => {
                state
                    .topics
                    .write()
                    .entry(topic.clone())
                    .or_insert_with(|| Topic::new(topic));

                peer.send(Transaction {
                    correlation,
                    exchange: Exchange::Response(Response::Ok),
                });
            }
            Request::Subscribe { topic: topic_name } => {
                let mut topics = state.topics.write();

                let response = if let Some(topic) = topics.get_mut(&topic_name) {
                    topic.subscribe(peer.clone(), correlation);
                    peer.subscribe(topic_name);

                    Response::Ok
                } else {
                    Response::UnknownTopic
                };

                peer.send(Transaction {
                    correlation,
                    exchange: Exchange::Response(response),
                });
            }
            Request::Unsubscribe { topic: topic_name } => {
                let mut topics = state.topics.write();

                let response = if let Some(topic) = topics.get_mut(&topic_name) {
                    topic.unsubscribe(&peer);
                    peer.unsubscribe(&topic_name);

                    Response::Ok
                } else {
                    Response::UnknownTopic
                };

                peer.send(Transaction {
                    correlation,
                    exchange: Exchange::Response(response),
                });
            }
            Request::Ping => {
                peer.send(Transaction {
                    correlation,
                    exchange: Exchange::Response(Response::Ok),
                });
            }
        };

        Ok(())
    })
}

fn sender(
    rx: impl Stream<Item = Transaction, Error = ()>,
    writer: impl Sink<SinkItem = Transaction, SinkError = Error>,
    peer: Arc<Peer>,
) -> impl Future<Item = (), Error = ()> {
    writer
        .sink_map_err(|err| eprintln!("Failed to write: {}", err))
        .send_all(rx.inspect(move |transaction| println!("< {}: {:?}", peer.addr, transaction)))
        .map(|_| ())
}

pub fn start() {
    let addr = "127.0.0.1:3001".parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let state = Arc::new(State::new());

    let server = socket
        .incoming()
        .map_err(|e| eprintln!("Failed to accept socket: {:?}", e))
        .for_each(move |stream| {
            process(stream, state.clone());
            Ok(())
        });

    tokio::run(server);
}
