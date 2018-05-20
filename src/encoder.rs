use std::io::Write;
use std::fmt::Write as FmtWrite;

use protocol::OutcomingMessage;

const LINE_CAPACITY: usize = 128;

pub struct Encoder<W: Write> {
    writer: W,
    line: String,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Encoder<W> {
        Encoder {
            writer,
            line: String::with_capacity(LINE_CAPACITY),
        }
    }

    pub fn write(&mut self, message: OutcomingMessage) {
        match message {
            OutcomingMessage::Ok => {
                self.writer.write(b"OK\n").unwrap();
            },
            OutcomingMessage::Err(error) => {
                self.line.clear();

                write!(&mut self.line, "{}\n", error).unwrap();

                self.writer.write(self.line.as_bytes()).unwrap();
            },
            OutcomingMessage::Data { topic_name, payload } => {
                self.line.clear();

                write!(&mut self.line, "{} {} ", topic_name, payload.len()).unwrap();

                // TODO: include \n in the buffer to avoid extra syscalls.

                self.writer.write(self.line.as_bytes()).unwrap();
                self.writer.write(&payload).unwrap();
                self.writer.write(b"\n").unwrap();
            }
        }
    }
}
