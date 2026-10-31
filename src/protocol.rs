/// Size of `fuse_in_header`, which starts every request from the kernel.
pub const IN_HEADER_LEN: usize = 40;

/// The header at the start of every request (`struct fuse_in_header` in fuse.h).
#[derive(Debug, PartialEq, Eq)]
pub struct InHeader {
    pub len: u32,
    pub opcode: u32,
    pub unique: u64,
    pub nodeid: u64,
    pub uid: u32,
    pub gid: u32,
    pub pid: u32,
}

impl InHeader {
    /// Reads the header from the start of `bytes`. Returns `None` if `bytes` is too short.
    pub fn parse(bytes: &[u8]) -> Option<InHeader> {
        if bytes.len() < IN_HEADER_LEN {
            return None;
        }
        Some(InHeader {
            len: read_u32(bytes, 0)?,
            opcode: read_u32(bytes, 4)?,
            unique: read_u64(bytes, 8)?,
            nodeid: read_u64(bytes, 16)?,
            uid: read_u32(bytes, 24)?,
            gid: read_u32(bytes, 28)?,
            pid: read_u32(bytes, 32)?,
        })
    }
}

/// Reads a little-endian u32 at `offset`, or `None` if out of bounds.
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let chunk = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(chunk.try_into().ok()?))
}

/// Reads a little-endian u64 at `offset`, or `None` if out of bounds.
fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let chunk = bytes.get(offset..offset + 8)?;
    Some(u64::from_le_bytes(chunk.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    // INIT request captured on om-minipc (Linux, FUSE 7.45).
    const INIT_REQUEST: [u8; 104] = [
        0x68, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, // len = 104, opcode = 26 (INIT)
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // unique = 2
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // nodeid = 0
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // uid, gid
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // pid, total_extlen, padding
        0x07, 0x00, 0x00, 0x00, 0x2d, 0x00, 0x00, 0x00, // major = 7, minor = 45
        0x00, 0x00, 0x02, 0x00, 0xfb, 0xff, 0xff, 0x73, // max_readahead, flags
        0xff, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // flags2, unused...
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn parses_init_header() {
        let header = InHeader::parse(&INIT_REQUEST).unwrap();
        assert_eq!(
            header,
            InHeader { len: 104, opcode: 26, unique: 2, nodeid: 0, uid: 0, gid: 0, pid: 0 }
        );
    }

    #[test]
    fn rejects_short_input() {
        assert_eq!(InHeader::parse(&INIT_REQUEST[..IN_HEADER_LEN - 1]), None);
    }
}
