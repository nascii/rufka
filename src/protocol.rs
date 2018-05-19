/*
   CREATE <topic_name>\n

   SUB <topic_name>\n

   <size><payload>\n

   PUB <topic_name>\n
   <size>\n
   <msg>\n
   <size>\n
   <msg>\n
*/

use std::rc::Rc;

use bufpool::Buffer;

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
}

#[derive(Debug)]
pub enum OutcomingMessage {
    Ok,
    Error {
    },
    Data {
        payload: Rc<Buffer>,
    },
}
