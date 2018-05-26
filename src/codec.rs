use std::str::{self, FromStr};

use bytes::{BufMut, Bytes, BytesMut};
use nom::digit;
use tokio_io::codec::{Decoder, Encoder};

use errors::{Error, ErrorKind, Result};
use protocol::{IncomingMessage, OutcomingMessage};

/*
 * CREATE <topic>\r\n
 *
 * SUB <topic>\r\n
 *
 * UNSUB <topic>\r\n
 *
 * PUB <topic> <size>\r\n<payload>\r\n
 *
 * ---
 *
 * Ok\r\n
 *
 * Invalid command
 *
 *
 *
 * <topic> <size>\r\n<payload>\r\n
 */
pub struct TextCodec {
    topic_name: Option<Bytes>,
    payload_size: usize,
}

impl TextCodec {
    pub fn new() -> TextCodec {
        TextCodec {
            topic_name: None,
            payload_size: 0,
        }
    }
}

impl Encoder for TextCodec {
    type Item = OutcomingMessage;
    type Error = Error;

    fn encode(&mut self, message: OutcomingMessage, dest: &mut BytesMut) -> Result<()> {
        match message {
            OutcomingMessage::Ok => {
                dest.extend_from_slice(b"Ok\r\n");
            }
            OutcomingMessage::InvalidCommand => {
                dest.extend_from_slice(b"Invalid command\r\n");
            }
            OutcomingMessage::UnknownTopic => {
                dest.extend_from_slice(b"Unknown topic\r\n");
            }
            OutcomingMessage::Data {
                topic_name,
                payload,
            } => {
                dest.reserve(
                    topic_name.len() +  // topic
                    1 +                 // space
                    10 +                // size (u32)
                    2 +                 // newline
                    payload.len() +     // payload
                    2, // newline
                );

                dest.put(topic_name);
                dest.put(" ");
                dest.put(format!("{}", payload.len()));
                dest.put("\r\n");
                dest.put(payload);
                dest.put("\r\n");
            }
        }

        Ok(())
    }
}

impl Decoder for TextCodec {
    type Item = IncomingMessage;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<IncomingMessage>> {
        if self.payload_size == 0 {
            let newline_offset = match buf.iter().position(|b| *b == b'\n') {
                Some(offset) => offset,
                None => return Ok(None),
            };

            let line = buf.split_to(newline_offset + 1);
            //let line = &line[..line.len() - 1];
            let line = str::from_utf8(&line).expect("invalid utf8 data");

            // Allow empty lines.
            if line.trim().is_empty() {
                return Ok(None);
            }

            let cmd = command(line)
                .map(|(_, cmd)| cmd)
                .map_err(|_| Error::from(ErrorKind::InvalidCommand))?;

            match cmd {
                Command::Create(topic_name) => {
                    return Ok(Some(IncomingMessage::Create {
                        topic_name: Bytes::from(topic_name),
                    }))
                }
                Command::Sub(topic_name) => {
                    return Ok(Some(IncomingMessage::Subscribe {
                        topic_name: Bytes::from(topic_name),
                    }))
                }
                Command::Unsub(topic_name) => {
                    return Ok(Some(IncomingMessage::Unsubscribe {
                        topic_name: Bytes::from(topic_name),
                    }))
                }
                Command::Pub(topic_name, size) => {
                    self.topic_name = Some(Bytes::from(topic_name));
                    self.payload_size = size as usize;
                }
            }
        }

        Ok(if buf.len() >= self.payload_size {
            let message = IncomingMessage::Publish {
                topic_name: self.topic_name.take().unwrap(),
                payload: buf.split_to(self.payload_size).freeze(),
            };

            self.payload_size = 0;

            Some(message)
        } else {
            None
        })
    }
}

#[derive(Debug, PartialEq)]
enum Command<'a> {
    Create(&'a str),
    Sub(&'a str),
    Unsub(&'a str),
    Pub(&'a str, u32),
}

named!(command<&str, Command>, terminated!(
    switch!(
        terminated!(command_name, char!(' ')),
        "PUB" => call!(pub_command) |
        "SUB" => call!(sub_command) |
        "UNSUB" => call!(unsub_command) |
        "CREATE" => call!(create_command)
    ),
    tag!("\r\n")
));

named!(command_name<&str, &str>, take_while!(|c| 'A' <= c && c <= 'Z'));

// CREATE topic-name
//        ^
named!(create_command<&str, Command>, do_parse!(
    topic_name: is_not!(" \r") >>
    (Command::Create(topic_name))
));

// SUB topic-name
//     ^
named!(sub_command<&str, Command>, do_parse!(
    topic_name: is_not!(" \r") >>
    (Command::Sub(topic_name))
));

// UNSUB topic-name
//     ^
named!(unsub_command<&str, Command>, do_parse!(
    topic_name: is_not!(" \r") >>
    (Command::Unsub(topic_name))
));

// PUB topic-name 42
//     ^
named!(pub_command<&str, Command>, do_parse!(
   topic_name: is_not!(" ") >>
   size: map_res!(preceded!(char!(' '), digit), u32::from_str) >>
   (Command::Pub(topic_name, size))
));

#[test]
fn it_parse_pub_command() {
    assert_eq!(
        command("PUB topic-name 42\r\n"),
        Ok(("", Command::Pub("topic-name", 42)))
    );
}

#[test]
fn it_parse_sub_command() {
    assert_eq!(
        command("SUB topic-name\r\n"),
        Ok(("", Command::Sub("topic-name")))
    );
}

#[test]
fn it_parse_unsub_command() {
    assert_eq!(
        command("UNSUB topic-name\r\n"),
        Ok(("", Command::Unsub("topic-name")))
    );
}

#[test]
fn it_parse_create_command() {
    assert_eq!(
        command("CREATE topic-name\r\n"),
        Ok(("", Command::Create("topic-name")))
    );
}
