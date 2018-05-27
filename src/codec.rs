use std::io::Cursor;

use bincode::Config;
use bytes::{Buf, BytesMut};
use tokio_io::codec::{Decoder, Encoder};

use errors::{Error, Result};
use protocol::{self, Container, Transaction};

pub struct Codec {
    config: Config,
}

impl Codec {
    pub fn new() -> Codec {
        Codec {
            config: protocol::config(),
        }
    }
}

impl Encoder for Codec {
    type Item = Transaction;
    type Error = Error;

    fn encode(&mut self, transaction: Transaction, buffer: &mut BytesMut) -> Result<()> {
        let container = Container {
            size: self.config.serialized_size(&transaction)? as i32,
            transaction: transaction,
        };

        self.config
            .serialize_into(Cursor::new(&mut **buffer), &container)
            .map_err(Error::from)
    }
}

impl Decoder for Codec {
    type Item = Transaction;
    type Error = Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Transaction>> {
        if buffer.len() <= 4 {
            return Ok(None);
        }

        let size = Cursor::new(&buffer).get_i32_be();

        if (buffer.len() as i32) < size {
            return Ok(None);
        }

        let transaction = self.config.deserialize(&buffer.split_to(size as usize))?;

        Ok(Some(transaction))
    }
}
