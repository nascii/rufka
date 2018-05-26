#[macro_use]
extern crate error_chain;
extern crate parking_lot;
#[macro_use]
extern crate nom;
extern crate bytes;
extern crate futures;
extern crate tokio;
extern crate tokio_io;

mod codec;
mod errors;
mod peer;
mod protocol;
mod server;
mod state;
mod topic;

fn main() {
    let srv = server::create();

    tokio::run(srv);
}
