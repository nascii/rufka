use std::sync::{RwLock, Arc};
use std::collections::HashMap;

use crossbeam_channel::{self as channel, Sender, Receiver};
use slab::Slab;

use bufpool::Buffer;
use protocol::OutcomingMessage;

const CLIENTS_CAPACITY: usize = 64;
const TOPICS_CAPACITY: usize = 32;

lazy_static! {
    static ref CLIENTS: RwLock<Slab<Client>> =
        RwLock::new(Slab::with_capacity(CLIENTS_CAPACITY));

    static ref TOPICS: RwLock<HashMap<String, Vec<ClientToken>>> =
        RwLock::new(HashMap::with_capacity(TOPICS_CAPACITY));
}

#[derive(Debug, PartialEq)]
pub struct ClientToken(usize);

impl ClientToken {
    fn clone(&self) -> ClientToken {
        ClientToken(self.0)
    }
}

struct Client {
    sender: Sender<OutcomingMessage>,
    subscriptions: Vec<String>,
}

pub fn connect() -> (ClientToken, Receiver<OutcomingMessage>) {
    let mut clients = CLIENTS.write().unwrap();

    let (sender, receiver) = channel::unbounded();

    let client = Client {
        sender,
        subscriptions: Vec::with_capacity(2),
    };

    let key = clients.insert(client);
    let token = ClientToken(key);

    (token, receiver)
}

pub fn create(token: &ClientToken, topic_name: String) {
    {
        let mut topics = TOPICS.write().unwrap();

        topics.entry(topic_name).or_insert_with(Vec::new);
    }

    let clients = CLIENTS.read().unwrap();

    clients[token.0].sender.send(OutcomingMessage::Ok).unwrap();
}

pub fn subscribe(token: &ClientToken, topic_name: &str) {
    let mut topics = TOPICS.write().unwrap();

    let message = if let Some(topic) = topics.get_mut(topic_name) {
        topic.push(token.clone());

        OutcomingMessage::Ok
    } else {
        OutcomingMessage::UnknownTopic(topic_name.to_string())
    };

    drop(topics);

    let mut clients = CLIENTS.write().unwrap();
    let client = clients.get_mut(token.0).unwrap();

    client.subscriptions.push(topic_name.to_string());
    client.sender.send(message).unwrap();
}

pub fn publish(token: &ClientToken, topic_name: &str, payload: Buffer) {
    let topics = TOPICS.read().unwrap();
    let clients = CLIENTS.read().unwrap();

    let message = if let Some(topic) = topics.get(topic_name) {
        let buffer = Arc::new(payload);

        for token in topic {
            let message = OutcomingMessage::Data {
                topic_name: topic_name.to_string(),
                payload: buffer.clone(),
            };

            clients[token.0].sender.send(message).unwrap();
        }

        OutcomingMessage::Ok
    } else {
        OutcomingMessage::UnknownTopic(topic_name.to_string())
    };

    drop(topics);

    clients[token.0].sender.send(message).unwrap();
}

pub fn disconnect(token: ClientToken) {
    let client = {
        let mut clients = CLIENTS.write().unwrap();

        clients.remove(token.0)
    };

    let mut topics = TOPICS.write().unwrap();

    for topic_name in client.subscriptions {
        // TODO: use `swap_remove` instead.
        topics.get_mut(&topic_name)
            .unwrap()
            .retain(|stored| stored != &token);
    }
}