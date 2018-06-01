use bincode::{self, Config};
use bytes::Bytes;

#[derive(Serialize, Deserialize, Debug)]
pub struct Container {
    pub size: i32,
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub correlation: CorrelationId,
    pub exchange: Exchange,
}

pub type CorrelationId = i32;

#[derive(Serialize, Deserialize, Debug)]
pub enum Exchange {
    Request(Request),
    Response(Response),
    Message(Message),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Ping,
    Create {
        topic: Bytes,
    },
    Subscribe {
        topic: Bytes,
    },
    Unsubscribe {
        topic: Bytes,
    },
    Publish {
        topic: Bytes,
        key: Bytes,
        value: Bytes,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ok,
    InvalidCommand,
    UnknownTopic,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub offset: i64,
    pub key: Bytes,
    pub value: Bytes,
}

pub fn config() -> Config {
    let mut config = bincode::config();

    config.limit(i32::max_value() as u64).big_endian();

    config
}
