use std::sync::Arc;

use bytes::Bytes;

use peer::Peer;
use protocol::OutcomingMessage;

pub struct Topic {
    name: Bytes,
    subscribers: Vec<Arc<Peer>>,
}

impl Topic {
    pub fn new(name: Bytes) -> Topic {
        Topic {
            name,
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, peer: Arc<Peer>) {
        self.subscribers.push(peer);
    }

    pub fn unsubscribe(&mut self, peer: &Arc<Peer>) {
        if let Some(idx) = self
            .subscribers
            .iter()
            .position(|sub| Arc::ptr_eq(sub, peer))
        {
            self.subscribers.swap_remove(idx);
        }
    }

    pub fn send(&self, payload: Bytes) {
        for peer in &self.subscribers {
            peer.send(OutcomingMessage::Data {
                topic_name: self.name.clone(),
                payload: payload.clone(),
            });
        }
    }
}
