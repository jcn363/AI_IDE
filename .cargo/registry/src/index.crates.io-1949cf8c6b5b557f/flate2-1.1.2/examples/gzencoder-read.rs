use flate2::{
    read::GzEncoder,
    Compression,
};
use std::{
    io,
    io::prelude::*,
};

// Print the GZ compressed representation of hello world
fn main() {
    println!("{:?}", gzencoder_read_hello_world().unwrap());
}

// Return a vector containing the GZ compressed version of hello world
fn gzencoder_read_hello_world() -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let c = b"hello world";
    let mut z = GzEncoder::new(&c[..], Compression::fast());
    z.read_to_end(&mut result)?;
    Ok(result)
}
