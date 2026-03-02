// Helper functions to help monitor and log state of kernel

use std::thread;
use std::time::Duration;


// mainly for debugging
pub fn sleep_s_with_log(seconds: u64, msg: &str) {
    println!("{msg}");
    println!("Sleeping for {seconds} seconds");
    thread::sleep(Duration::from_secs(seconds));
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
