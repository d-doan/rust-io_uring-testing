mod monitor;
mod ghost_memory;
mod covert_channel;
mod transmitter;
mod receiver;

use std::env;

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


fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("--help");
    
    match mode {
        "g"     | "ghost" => ghost_memory::demo(),
        "c"     | "covert" => covert_channel::demo(),
        "tx"    | "transmit" => transmitter::demo(),
        "rx"    | "receive" => receiver::demo(),
        _ => {
            println!("Usage: cargo run -- [MODE]");
            println!("  g,  ghost  Shows memory desync between RAM and process");
            println!("  c,  covert Shows bi-directional covert channel");
            println!("  tx, transmit Initiates covert transmission. Start rx mode in another terminal to enable IPC through RAM");
            println!("  rx, receive Initiates covert reception. Start tx mode in another terminal first to enable IPC through RAM"); 
        }
    }
}
