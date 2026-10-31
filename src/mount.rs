#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::io;

/// Calls the Linux mount() syscall to mount a FUSE filesystem on `mountpoint`.
#[cfg(target_os = "linux")]
pub fn mount_fuse(mountpoint: &str, fd: i32) -> io::Result<()> {
    let source = CString::new("rufuse")?;
    let target = CString::new(mountpoint)?;
    let fstype = CString::new("fuse")?;

    // fd, rootmode, user_id and group_id are required by the kernel.
    // allow_other: we mount as root, but normal users need access too.
    let options = format!("fd={},rootmode=40000,user_id=0,group_id=0,allow_other", fd);
    let data = CString::new(options)?;

    // SAFETY: all pointers point to valid C strings that live until mount() returns.
    let result = unsafe {
        libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            fstype.as_ptr(),
            0,
            data.as_ptr() as *const libc::c_void,
        )
    };

    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Mounting is only implemented for Linux so far.
#[cfg(not(target_os = "linux"))]
pub fn mount_fuse(_mountpoint: &str, _fd: i32) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "mounting is only supported on Linux",
    ))
}
