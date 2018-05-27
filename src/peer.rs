use std::mem;
use std::net::SocketAddr;

use bytes::Bytes;
use futures::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use parking_lot::Mutex;

use protocol::Transaction;

pub struct Peer {
    pub addr: SocketAddr,
    sender: UnboundedSender<Transaction>,
    subscriptions: Mutex<Vec<Bytes>>,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> (Peer, UnboundedReceiver<Transaction>) {
        let (sender, receiver) = mpsc::unbounded();

        let peer = Peer {
            addr,
            sender,
            subscriptions: Mutex::new(Vec::new()),
        };

        (peer, receiver)
    }

    pub fn subscribe(&self, topic_name: Bytes) {
        self.subscriptions.lock().push(topic_name);
    }

    pub fn unsubscribe(&self, topic_name: &Bytes) {
        let mut subscriptions = self.subscriptions.lock();

        if let Some(idx) = subscriptions.iter().position(|sub| *sub == topic_name) {
            subscriptions.swap_remove(idx);
        }
    }

    pub fn unsubscribe_all(&self) -> Vec<Bytes> {
        let mut subscriptions = self.subscriptions.lock();

        mem::replace(&mut *subscriptions, Vec::new())
    }

    pub fn send(&self, transaction: Transaction) {
        self.sender.unbounded_send(transaction).unwrap()
    }
}
