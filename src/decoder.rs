use std::io::{Read, BufRead, BufReader};
use std::net::TcpStream;
use std::str::FromStr;

use nom::digit;

use bufpool;
use protocol::IncomingMessage;

const LINE_CAPACITY: usize = 128;

pub struct Decoder {
    reader: BufReader<TcpStream>,
    line: String,
}

impl Decoder {
    pub fn new(stream: TcpStream) -> Decoder {
        Decoder {
            reader: BufReader::new(stream),
            line: String::with_capacity(LINE_CAPACITY),
        }
    }

    fn read_line(&mut self) -> &str {
        self.line.clear();

        while {
            self.reader.read_line(&mut self.line).unwrap();

            if self.line.is_empty() {
                return "";
            }

            self.line.trim().is_empty()
        } {}

        &self.line.trim_left_matches("\r\n")
    }
}

impl Iterator for Decoder {
    type Item = IncomingMessage;

    fn next(&mut self) -> Option<IncomingMessage> {
        let mut message = {
            let line = self.read_line();

            if line.is_empty() {
                return None;
            }

            parse_message(&line)
        };

        if let IncomingMessage::Publish { ref mut payload, .. } = message {
            self.reader.read_exact(payload).unwrap();
        }

        Some(message)
    }
}

fn parse_message(line: &str) -> IncomingMessage {
    match parse_command(line).map(|(_, cmd)| cmd) {
        Ok(Command::Create(topic_name)) =>
            IncomingMessage::Create {
                topic_name: topic_name.to_string(),
            },
        Ok(Command::Sub(topic_name)) =>
            IncomingMessage::Subscribe {
                topic_name: topic_name.to_string(),
            },
        Ok(Command::Pub(topic_name, size)) => {
            let mut buffer = bufpool::get_buffer(size as usize);

            debug_assert!(buffer.len() as u32 == size);

            IncomingMessage::Publish {
                topic_name: topic_name.to_string(),
                payload: buffer,
            }
        },
        Err(_) => IncomingMessage::Invalid,
    }
}

#[derive(Debug, PartialEq)]
enum Command<'a> {
    Create(&'a str),
    Sub(&'a str),
    Pub(&'a str, u32),
}

named!(parse_command<&str, Command>, alt_complete!(
    do_parse!(
        tag!("PUB") >>
        topic_name: preceded!(char!(' '), is_not!(" ")) >>
        size: map_res!(
            delimited!(char!(' '), digit, tag!("\r\n")),
            u32::from_str
        ) >>
        (Command::Pub(topic_name, size))
    ) |
    do_parse!(
        tag!("SUB") >>
        topic_name: delimited!(char!(' '), is_not!(" \r"), tag!("\r\n")) >>
        (Command::Sub(topic_name))
    ) |
    do_parse!(
        tag!("CREATE") >>
        topic_name: delimited!(char!(' '), is_not!(" \r"), tag!("\r\n")) >>
        (Command::Create(topic_name))
    )
));

#[test]
fn it_parse_pub_command() {
    assert_eq!(parse_command("PUB topic-name 42\r\n"), Ok(("", Command::Pub("topic-name", 42))));
}

#[test]
fn it_parse_sub_command() {
    assert_eq!(parse_command("SUB topic-name\r\n"), Ok(("", Command::Sub("topic-name"))));
}

#[test]
fn it_parse_create_command() {
    assert_eq!(parse_command("CREATE topic-name\r\n"), Ok(("", Command::Create("topic-name"))));
}
