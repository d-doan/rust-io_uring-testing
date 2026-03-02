// Transmits messages to another process using io_uring by passing the same fd
// Allows us to bypass any monitored channel of communication and talk directly through the
// pinned RAM which we can perform IO with using io_uring
// This connection is not shown in the process' Virtual Memory since we dropped that buffer

use crate::monitor;

use io_uring::{opcode, types, IoUring, Parameters};
use nix::sys::socket::{ControlMessage, MsgFlags, sendmsg};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::thread;
use std::time::Duration;

pub fn demo() {
    let mut ring = IoUring::new(8).expect("failed to create ring");
    let size = 4096;
    let (ptr, v) = monitor::prepare_ghost_buffer(&mut ring, size, None);

    let ptr_addr = ptr as u64;
    println!("Target address: 0x{:x}", ptr_addr);
    drop(v); 
   
    // setup socket for handover
    let socket_path = "/tmp/ipc.sock";
    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path).unwrap();
    
    println!("[Tx] Waiting for Rx to connect");
    let (stream, _) = listener.accept().unwrap();
    println!("[Tx] Rx connected. Passing the Key (FD), Map (Ptr), and Params...");

    // Extract parameters to send over socket
    let params = ring.params();
    let params_bytes = unsafe {
        std::slice::from_raw_parts(
            params as *const Parameters as *const u8,
            std::mem::size_of::<Parameters>()
        )
    };
    let addr_bytes = ptr_addr.to_ne_bytes();
    
    let iov = [
        std::io::IoSlice::new(&addr_bytes),
        std::io::IoSlice::new(params_bytes),
    ];
    let fds = [ring.as_raw_fd()];
    let cmsg = [ControlMessage::ScmRights(&fds)];
    
    sendmsg::<()>(stream.as_raw_fd(), &iov, &cmsg, MsgFlags::empty(), None).unwrap();
    drop(stream); 

    // injection loop
    let in_path = "/tmp/ghost_input.txt";
    let mut f_in = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(in_path).unwrap();
    let in_fd = types::Fd(f_in.as_raw_fd());

    let mut count = 0;
    loop {
        count += 1;
      
        // zero out buffer
        f_in.seek(SeekFrom::Start(0)).unwrap();
        f_in.write_all(&[0u8; 32]).unwrap();
        let clear_op = opcode::ReadFixed::new(in_fd, ptr, 32, 0).build();
        unsafe { ring.submission().push(&clear_op).unwrap(); }
        ring.submit_and_wait(1).unwrap();
        while let Some(_) = ring.completion().next() {}

        // write actual message
        let msg = format!("SECRET_PULSE_{}", count);
        f_in.seek(SeekFrom::Start(0)).unwrap();
        f_in.write_all(msg.as_bytes()).unwrap();
        f_in.set_len(msg.len() as u64).unwrap();

        let op = opcode::ReadFixed::new(in_fd, ptr, msg.len() as u32, 0).build();
        unsafe { ring.submission().push(&op).unwrap(); }
        println!("Writing data from {} to RAM using io_uring", in_path);
        ring.submit_and_wait(1).unwrap();
        while let Some(_cqe) = ring.completion().next() {} 

        println!("[Tx] Injected: {}", msg);
        thread::sleep(Duration::from_millis(2000));
    }
}
