use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::fd::AsRawFd;

use rufuse::mount::mount_fuse;
use rufuse::protocol::InHeader;

fn main() -> io::Result<()> {
    let mut dev = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/fuse")?;

    let fd = dev.as_raw_fd();
    println!("opened /dev/fuse, fd = {}", fd);

    mount_fuse("/tmp/mnt", fd)?;
    println!("mounted /tmp/mnt");

    // The kernel rejects reads into small buffers; libfuse uses about this size.
    let mut buf = vec![0u8; 1024 * 1024 + 4096];

    // The first message after mounting is always INIT.
    let n = dev.read(&mut buf)?;
    println!("read {} bytes:", n);
    println!("{:#?}", InHeader::parse(&buf[..n]));

    // Exiting closes dev, which aborts the FUSE connection.
    println!("press Enter to exit");
    io::stdin().read_line(&mut String::new())?;

    Ok(())
}
