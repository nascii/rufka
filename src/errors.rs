use std::io;

use bincode;

error_chain! {
    foreign_links {
        Io(io::Error);
        Bincode(bincode::Error);
    }

    // TODO: wrap into custom errors.
}
