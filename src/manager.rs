use std::sync::{Mutex, Arc};
use std::sync::mpsc::Sender;
use std::collections::HashMap;

use bufpool::Buffer;

lazy_static! {
    static ref TOPICS: Mutex<HashMap<String, Vec<Sender<Arc<Buffer>>>>> =
        Mutex::new(HashMap::new());
}

pub fn create(topic_name: String) {
    let mut topics = TOPICS.lock().unwrap();

    topics.entry(topic_name).or_insert_with(Vec::new);
}

pub fn subscribe(topic_name: &str, tx: Sender<Arc<Buffer>>) {
    let mut topics = TOPICS.lock().unwrap();

    topics.get_mut(topic_name).unwrap().push(tx);
}

pub fn publish(topic_name: &str, payload: Buffer) {
    let topics = TOPICS.lock().unwrap();

    let topic = &topics[topic_name];

    let buffer = Arc::new(payload);

    for tx in topic {
        tx.send(buffer.clone()).unwrap();
    }
}
