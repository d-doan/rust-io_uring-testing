// Cmdline tool that demonstrates the following:
// Io_uring + dropping/unmapping memory lets us desync kernel from process Virtual Memory
// Explanation in README

mod monitor;
mod ghost_memory;
mod covert_channel;
mod transmitter;
mod receiver;
mod dos_single;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("--help");

    match mode {
        "g"     | "ghost" => ghost_memory::demo(),
        "c"     | "covert" => covert_channel::demo(),
        "tx"    | "transmit" => transmitter::demo(),
        "rx"    | "receive" => receiver::demo(),
        "sd"    | "singleDoS" => dos_single::demo(),
        _ => {
            println!("Usage: cargo run -- [MODE]");
            println!("  g,  ghost  Shows memory desync between RAM and process");
            println!("  c,  covert Shows bi-directional covert channel");
            println!("  tx, transmit Initiates covert transmission. Start rx mode in another terminal to enable IPC through RAM");
            println!("  rx, receive Initiates covert reception. Start tx mode in another terminal first to enable IPC through RAM");
            println!("  sd, uses a single process to create ghost memory in order to have a DoS attack");
        }
    }
}
