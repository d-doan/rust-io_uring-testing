// Cmdline tool that demonstrates the following:
// Io_uring + dropping/unmapping memory lets us desync kernel from process Virtual Memory
// Explanation below

// drop() or munmap() deletes the virtual memory area for that address range and clears the
// corresponding entries in the page tables
// This rightfully will triggerr a segfault if we try to directly access that memory

// However, when we register the buffer with io_uring, the kernel increases the ref count of the
// physical pages, so after 'freeing' the memory in our program, the kernel doesn't release the RAM

// We can exploit this by issuing ReadFixed or WriteFixed (or more) commands to io_uring
// io_uring does not look at page tables but directly to the physical address we pinned earlier
// enabling the kernel to do i/o for those pages even though they are invisible to the CPU's memory
// checks

// This behavior allow us to achieve the following
// 1. Evasion - since memory is unmapped from the VM, utilities such as ps, top, show our process to
//    be using 0 bytes of that RAM. We can manipulate this to potentially run many of these
//    processes to perform a DoS attack since the adminstrative program will throttle/kill program's
//    that it thinks uses a significant portion of memory (which we 'officially' don't use much)
//    Single process: -- ghost
//
// 2. Covert Channel - we can share the io_uring FD with another process (or fork setup
//    information in) which allows both process to be able to read and write to the same physical
//    RAM without either process having that RAM mapped in the virtual address space. The kernel
//    should have no way of knowing that these two processes are communicating and can't monitor it.
//    This may be used for some attacks etc.
//    Within same process: -- covert
//    IPC: --tx, --rx
//
// 3. This may enable Use after Free attacks if we exhaust enough of the RAM that the kernel
//    reclaims and reallocates the pinned pages as we are still able to modify the page. 
//    This has not been personally tested yet.

mod monitor;
mod ghost_memory;
mod covert_channel;
mod transmitter;
mod receiver;

use std::env;

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
