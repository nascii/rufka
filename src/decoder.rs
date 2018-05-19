use std::io::Read;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

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

        self.line.trim()
    }
}

impl Iterator for Decoder {
    type Item = IncomingMessage;

    fn next(&mut self) -> Option<IncomingMessage> {
        let (cmd, arg) = {
            let mut it = self.read_line().split_whitespace();

            let cmd = match it.next() {
                Some(x) => x,
                None => return None,
            };

            (cmd.to_string(), it.next().unwrap().to_string())
        };

        Some(match cmd.as_str() {
            "CREATE" => {
                IncomingMessage::Create {
                    topic_name: arg,
                }
            },
            "SUB" => {
                IncomingMessage::Subscribe {
                    topic_name: arg,
                }
            },
            "PUB" => {
                let size: usize = self.read_line().parse().unwrap();

                let mut buffer = bufpool::get_buffer(size);

                debug_assert!(buffer.len() == size);

                self.reader.read_exact(&mut buffer).unwrap();

                IncomingMessage::Publish {
                    topic_name: arg,
                    payload: buffer,
                }
            },
            _ => unreachable!(),
        })
    }
}
