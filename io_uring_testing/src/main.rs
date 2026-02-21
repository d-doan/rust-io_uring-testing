use io_uring::{opcode, types, IoUring};
use libc::{mmap, munmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE, iovec};
use std::os::unix::io::AsRawFd;
use std::{ptr, fs::File, io::Read};

// desync kernel from application
// pinning allows kernel to bypass standard page table checks
// io_uring implmentation doesn't synchronize virtual memory address with fixed buffer registration
// allows us to store data in a zombie page but be able to delete address mapping (munmap)
// deleting address mapping makes it so that malware scanners can't see it
// still able to use io_uring to write the zombie data even though it's 'deleted'
// functionally side channel storage

// enables Use after free?
// register buffer & munmap it
// wait for another process/own process to allocate same physical memory
// write to original fixed buffer index

// next steps:
// move towards safer rust
// check if registering + dropping vec<u8> works same way?

fn main() {
    unsafe {
        // allocate memory and store secret at addr
        let mut v = vec![0u8; 4096];
        let secret = b"secret zombie data";
        v[..secret.len()].copy_from_slice(secret);
        println!("wrote secret to memory: {:?}", std::str::from_utf8(secret).unwrap());

        let ptr = v.as_ptr();
        let len = v.len();

        // pin it with fixed buffer
        // pinning increments ref count on physical RAM
        let iov = iovec { iov_base: ptr as *mut _, iov_len: len };
        let mut ring = IoUring::new(8).expect("ring failed");
        ring.submitter().register_buffers(&[iov]).expect("register failed");

        // 'free' memory, memory should be gone in rust's eyes
        drop(v);

        let file = File::create("/tmp/zombie_output.txt").unwrap();
        let fd = types::Fd(file.as_raw_fd());
        println!("ask kernel to get secret from zombie page");

        // writes from the registered buffer to the file descriptor.
        // kernel thread is able to write our 'zombie data' even though process shouldn't have mapping
        let write_op = opcode::WriteFixed::new(fd, ptr, secret.len() as u32, 0).build();
        ring.submission().push(&write_op).expect("queue full");
        ring.submit_and_wait(1).unwrap();

        // check results
        let mut check_file = File::open("/tmp/zombie_output.txt").unwrap();
        let mut contents = String::new();
        check_file.read_to_string(&mut contents).unwrap();

        println!("\ncomparing expected versus what kernel read");
        println!("expected: {:?}", std::str::from_utf8(secret).unwrap());
        println!("actual:   {:?}", contents);

        if contents.as_bytes() == secret {
            println!("side channel storage enabled");
        } else {
            println!("data got lost or corrupted");
        }
    }
}
