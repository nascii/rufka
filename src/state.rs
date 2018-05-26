use std::collections::HashMap;

use bytes::Bytes;
use parking_lot::RwLock;

use topic::Topic;

#[derive(Default)]
pub struct State {
    pub topics: RwLock<HashMap<Bytes, Topic>>,
}

impl State {
    pub fn new() -> State {
        State {
            topics: RwLock::new(HashMap::new()),
        }
    }
}
