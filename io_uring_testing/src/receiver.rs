// Receives message from another process using io_uring 
// Allows us to bypass any monitored channel of communication and instead talk directly through
// pinned RAM which we have access to with io_uring
// This connection is not shown in the process' Virtual Memory since we dropped that buffer

use io_uring::{opcode, types, IoUring, Parameters};
use nix::sys::socket::{recvmsg, ControlMessageOwned, MsgFlags};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

pub fn demo() {
    println!("[Rx] Connecting to Tx...");
    let stream = UnixStream::connect("/tmp/ipc.sock").expect("Receiver is not running");

    // prepare buffer and Paramaters to be filled in by tx
    let mut addr_buf = [0u8; 8];
    let mut params = unsafe { std::mem::zeroed::<Parameters>() };
    let params_bytes = unsafe {
        std::slice::from_raw_parts_mut(
            &mut params as *mut Parameters as *mut u8,
            std::mem::size_of::<Parameters>()
        )
    };

    let mut iov = [
        std::io::IoSliceMut::new(&mut addr_buf),
        std::io::IoSliceMut::new(params_bytes),
    ];
    let mut cmsgspace = nix::cmsg_space!(RawFd);

    // receive msg from socket
    let msg = recvmsg::<()>(
        stream.as_raw_fd(),
        &mut iov,
        Some(&mut cmsgspace),
        MsgFlags::empty(),
    ).expect("Did not receive socket message");

    let cmsgs = msg.cmsgs().expect("Could not parse control messages");
    let mut fd: Option<RawFd> = None;

    for cmsg in cmsgs {
        if let ControlMessageOwned::ScmRights(fds) = cmsg && !fds.is_empty(){
            fd = Some(fds[0]);
            break;
        }
    }

    let fd = fd.expect("No FD was passed over the socket");
    let ptr_val = u64::from_ne_bytes(addr_buf);
    println!("[Rx] Adopting Ring FD: {}. Target Address: 0x{:x}", fd, ptr_val);

    // 'recreate ring'
    let mut ring = unsafe { IoUring::from_fd(fd, params) }.expect("Failed to adopt io_uring FD");

    let out_path = "/tmp/sniffed_stream.txt";
    let mut f_out = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .expect("Failed to create output file");
    let out_fd = types::Fd(f_out.as_raw_fd());

    thread::sleep(Duration::from_millis(1000));

    loop {

        let op = opcode::WriteFixed::new(out_fd, ptr_val as *const u8, 32, 0).build();
        
        unsafe {
            ring.submission().push(&op).expect("Submission queue is full");
        }
        println!("Writing from pinned RAM into {}", out_path);
        ring.submit_and_wait(1).expect("Kernel failed to process uring submission");

        while let Some(_cqe) = ring.completion().next() {}

        let mut buffer = vec![0u8; 32];
        f_out.seek(SeekFrom::Start(0)).unwrap();
        f_out.read_exact(&mut buffer).unwrap();
        
        let raw_str = String::from_utf8_lossy(&buffer);
        let clean_msg = raw_str.trim_matches(char::from(0));

        if !clean_msg.is_empty() {
            println!("[Rx] Extracted: \"{}\"", clean_msg);
        }
        
        // clear file for next iter
        f_out.set_len(0).unwrap();

        thread::sleep(Duration::from_millis(2000));
    } 
}
