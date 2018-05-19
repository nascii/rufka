#[macro_use]
extern crate lazy_static;

mod bufpool;
mod protocol;
mod server;
mod decoder;
mod manager;

fn main() {
    server::run();
}
