#[macro_use]
extern crate lazy_static;
extern crate crossbeam_channel;
extern crate slab;

mod bufpool;
mod protocol;
mod server;
mod decoder;
mod encoder;
mod manager;

fn main() {
    server::run();
}
