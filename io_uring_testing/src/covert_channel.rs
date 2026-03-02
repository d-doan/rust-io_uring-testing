// Demonstrate being able to use io_uring to read/write from a file that was dropped in rust
// If that memory location was accessed in rust it will segfault 
// We can use this to read/write memory that isn't associated with the process anymore

use crate::monitor;

use io_uring::{opcode, types, IoUring};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

pub fn demo() {
    let mut ring = IoUring::new(8).expect("ring failed");
    let size = 1024;
    let secret = b"FILLER";
    println!("Registering secret buffer with io_uring");
    let (ptr, v) = monitor::prepare_ghost_buffer(&mut ring, size, Some(secret));

    let prefix = std::str::from_utf8(&v[..secret.len()]).unwrap();
    println!("Buffer starts as: {}", prefix);

    println!("Dropping vector. Secret is unmapped now");
    // 'free' memory, memory should be gone in rust's eyes
    drop(v);

    // create new data file to write from
    let new_data = b"new update data";
    let mut src_file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("/tmp/ghost_input.txt").unwrap();
    src_file.write_all(new_data).unwrap();
    let src_fd = types::Fd(src_file.as_raw_fd());

    println!("Writing new data to unmapped buffer using io_uring");
    unsafe {
        libc::lseek(src_fd.0, 0, libc::SEEK_SET);
        let read_op = opcode::ReadFixed::new(src_fd, ptr, new_data.len() as u32, 0).build();
        ring.submission().push(&read_op).expect("queue full");
    }
    ring.submit_and_wait(1).unwrap();
    let _ = ring.completion().next();

    let out_file = OpenOptions::new().write(true).create(true).truncate(true).open("/tmp/ghost_output.txt").unwrap();
    let out_fd = types::Fd(out_file.as_raw_fd());

    println!("Retrieving current data from unmapped buffer to a new file");
    let write_op = opcode::WriteFixed::new(out_fd, ptr, new_data.len() as u32, 0).build();
    unsafe {
        ring.submission().push(&write_op).expect("queue full");
    }
    ring.submit_and_wait(1).unwrap();
    let _ = ring.completion().next();

    // check results
    let mut check_file = File::open("/tmp/ghost_output.txt").unwrap();
    let mut contents = String::new();
    check_file.read_to_string(&mut contents).unwrap();

    println!("Expected:  {:?}", std::str::from_utf8(new_data).unwrap());
    println!("Retrieved: {:?}", contents);

    if contents.as_bytes() == new_data {
        println!("bi-directional side channel enabled");
    } else {
        println!("data got lost or corrupted");
    }
    
}
