#![allow(unknown_lints)]

#[macro_use]
extern crate structopt;
#[macro_use]
extern crate error_chain;
extern crate bytes;
extern crate futures;
extern crate parking_lot;
extern crate tokio;
extern crate tokio_io;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

mod cli;
mod client;
mod codec;
mod errors;
mod peer;
mod protocol;
mod server;
mod state;
mod topic;

fn main() {
    cli::init();
}
