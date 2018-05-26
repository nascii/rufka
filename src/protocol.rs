/*
   CREATE <topic>\n

   SUB <topic>\n

   PUB <topic> <size>\n
   <msg>\n

   ---

   <topic> <size> <payload>\n
*/

use bytes::Bytes;

#[derive(Debug)]
pub enum IncomingMessage {
    Create { topic_name: Bytes },
    Subscribe { topic_name: Bytes },
    Publish { topic_name: Bytes, payload: Bytes },
}

#[derive(Debug)]
pub enum OutcomingMessage {
    Ok,
    InvalidCommand,
    UnknownTopic,
    Data { topic_name: Bytes, payload: Bytes },
}
