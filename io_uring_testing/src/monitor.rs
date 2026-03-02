// Helper functions to help monitor and log state of kernel
// Along with helper functions to setup io_uring state

use io_uring::IoUring;
use libc::iovec;

/// set up physical buffer in RAM and pin it
/// Returns the raw pointer and original Vec
pub fn prepare_ghost_buffer(
    ring: &mut IoUring,
    size: usize,
    secret: Option<&[u8]>
) -> (*mut u8, Vec<u8>) {
    let mut v = vec![0u8; size];
    let ptr = v.as_mut_ptr();

    let page_size = 4096;
    // touch every page to trigger lazy loading
    for i in (0..size).step_by(page_size) {
        v[i] = 1;
    }

    if let Some(s) = secret {
        let len = s.len().min(size);
        v[..len].copy_from_slice(&s[..len]);
    }
    unsafe {
        let iov = iovec {iov_base: ptr as *mut _, iov_len: size};
        ring.submitter().register_buffers(&[iov]).expect("register failed");
    }
    (ptr, v)
}

// logs process and system wide RAM stats 
pub fn log_memory_stats(label: &str) {
    // get process resident set size (RSS)
    let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
    let rss_pages: u64 = statm.split_whitespace().nth(1).unwrap().parse().unwrap();
    let rss_mb = (rss_pages * 4096) / 1024 / 1024;

    // get system free RAM
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap();
    let free_kb: u64 = meminfo.lines()
        .find(|line| line.starts_with("MemAvailable:"))
        .map(|line| line.split_whitespace().nth(1).unwrap().parse().unwrap())
        .unwrap_or(0);
    let free_mb = free_kb / 1024;

    println!("\n---- [ {} ] ----", label);
    println!("Process RSS:    {} MB", rss_mb);
    println!("System Available:   {} MB", free_mb);
    println!("--------------------------\n");
}
