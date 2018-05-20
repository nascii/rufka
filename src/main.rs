#[macro_use]
extern crate lazy_static;
extern crate crossbeam_channel;
extern crate slab;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate failure;

mod bufpool;
mod protocol;
mod server;
mod decoder;
mod encoder;
mod manager;
mod errors;

fn main() {
    server::run();
}
