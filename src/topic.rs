use std::sync::Arc;

use bytes::Bytes;

use peer::Peer;
use protocol::{CorrelationId, Exchange, Message, Transaction};

pub struct Topic {
    subscribers: Vec<Subscriber>,
}

struct Subscriber {
    peer: Arc<Peer>,
    correlation: CorrelationId,
}

impl Topic {
    pub fn new(_name: Bytes) -> Topic {
        Topic {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, peer: Arc<Peer>, correlation: CorrelationId) {
        self.subscribers.push(Subscriber { peer, correlation });
    }

    pub fn unsubscribe(&mut self, peer: &Arc<Peer>) {
        if let Some(idx) = self
            .subscribers
            .iter()
            .position(|sub| Arc::ptr_eq(&sub.peer, peer))
        {
            self.subscribers.swap_remove(idx);
        }
    }

    pub fn send(&self, message: Message) {
        for subscriber in &self.subscribers {
            subscriber.peer.send(Transaction {
                correlation: subscriber.correlation,
                exchange: Exchange::Message(message.clone()),
            });
        }
    }
}
