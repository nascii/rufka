use std::mem;
use std::sync::Mutex;
use std::ops::{Deref, DerefMut};

const POOL_CAPACITY: usize = 1024;

lazy_static! {
    static ref POOL: Mutex<Vec<Vec<u8>>> =
        Mutex::new(Vec::with_capacity(POOL_CAPACITY));
}

pub fn get_buffer(size: usize) -> Buffer {
    let mut list = POOL.lock().unwrap();

    let vec = match list.pop() {
        Some(mut vec) => {
            vec.resize(size, 0);
            vec
        },
        None => vec![0; size],
    };

    Buffer(vec)
}

fn put_buffer(buffer: Vec<u8>) {
    let mut list = POOL.lock().unwrap();

    list.push(buffer);
}

#[derive(Debug)]
pub struct Buffer(Vec<u8>);

impl Drop for Buffer {
    fn drop(&mut self) {
        let free_vec = mem::replace(&mut self.0, Vec::new());

        put_buffer(free_vec);
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
