/*
   CREATE <topic>\n

   SUB <topic>\n

   PUB <topic> <size>\n
   <msg>\n

   ---

   <topic> <size> <payload>\n
*/

use std::sync::Arc;

use bufpool::Buffer;
use errors::Error;

#[derive(Debug)]
pub enum IncomingMessage {
    Create {
        topic_name: String,     // TODO: slab
    },
    Subscribe {
        topic_name: String,
    },
    Publish {
        topic_name: String,
        payload: Buffer,
    },
    Invalid,
}

#[derive(Debug)]
pub enum OutcomingMessage {
    Ok,
    Err(Error),
    Data {
        topic_name: String,
        payload: Arc<Buffer>,
    },
}
