use std::sync::Arc;

use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use codec::TextCodec;
use errors::Error;
use peer::Peer;
use protocol::{IncomingMessage, OutcomingMessage};
use state::State;
use topic::Topic;

fn process(stream: TcpStream, state: Arc<State>) {
    let addr = stream.peer_addr().unwrap();

    println!("{}: connected", addr);

    let (writer, reader) = stream.framed(TextCodec::new()).split();

    let (peer, rx) = Peer::new(addr);

    let peer = Arc::new(peer);

    let receiving = receiver(state.clone(), reader, peer.clone());
    let sending = sender(state.clone(), rx, writer, peer.clone());

    // TODO: wait last message before closing.

    let connection = receiving.select(sending).then(move |_| {
        let mut topics = state.topics.write();

        for topic_name in peer.unsubscribe_all() {
            if let Some(topic) = topics.get_mut(&topic_name) {
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
    reader: impl Stream<Item = IncomingMessage, Error = Error>,
    peer: Arc<Peer>,
) -> impl Future<Item = (), Error = ()> {
    let peer_clone = peer.clone();

    let reader = reader.map_err(move |_| {
        let message = OutcomingMessage::InvalidCommand;
        println!("> {}: {:?}", peer_clone.addr, message);

        peer_clone.send(message);
        ()
    });

    reader.for_each(move |message| {
        println!("> {}: {:?}", peer.addr, message);

        match message {
            IncomingMessage::Publish {
                topic_name,
                payload,
            } => {
                let mut topics = state.topics.read();

                let response = if let Some(topic) = topics.get(&topic_name) {
                    topic.send(payload);

                    OutcomingMessage::Ok
                } else {
                    OutcomingMessage::UnknownTopic
                };

                peer.send(response);
            }
            IncomingMessage::Create { topic_name } => {
                state
                    .topics
                    .write()
                    .entry(topic_name.clone())
                    .or_insert_with(|| Topic::new(topic_name));

                peer.send(OutcomingMessage::Ok);
            }
            IncomingMessage::Subscribe { topic_name } => {
                let mut topics = state.topics.write();

                let response = if let Some(topic) = topics.get_mut(&topic_name) {
                    topic.subscribe(peer.clone());
                    peer.subscribe(topic_name);

                    OutcomingMessage::Ok
                } else {
                    OutcomingMessage::UnknownTopic
                };

                peer.send(response);
            }
        };

        Ok(())
    })
}

fn sender(
    _state: Arc<State>,
    rx: impl Stream<Item = OutcomingMessage, Error = ()>,
    writer: impl Sink<SinkItem = OutcomingMessage, SinkError = Error>,
    peer: Arc<Peer>,
) -> impl Future<Item = (), Error = ()> {
    writer
        .sink_map_err(|err| eprintln!("{}", err))
        .send_all(rx.inspect(move |message| println!("< {}: {:?}", peer.addr, message)))
        .map(|_| ())
}

pub fn create() -> impl Future<Item = (), Error = ()> {
    let addr = "127.0.0.1:3001".parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let state = Arc::new(State::new());

    socket
        .incoming()
        .map_err(|e| eprintln!("Failed to accept socket: {:?}", e))
        .for_each(move |stream| {
            process(stream, state.clone());
            Ok(())
        })
}
