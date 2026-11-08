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
            len: u32_le_at(bytes, 0)?,
            opcode: u32_le_at(bytes, 4)?,
            unique: u64_le_at(bytes, 8)?,
            nodeid: u64_le_at(bytes, 16)?,
            uid: u32_le_at(bytes, 24)?,
            gid: u32_le_at(bytes, 28)?,
            pid: u32_le_at(bytes, 32)?,
        })
    }
}

/// The body of an INIT request (`struct fuse_init_in` in fuse.h).
#[derive(Debug, PartialEq, Eq)]
pub struct InitIn {
    pub major: u32,
    pub minor: u32,
    pub max_readahead: u32,
    pub flags: u32,
    /// Feature bits 32–63. 0 if the kernel didn't send them (no `INIT_EXT`, or an older kernel).
    pub flags2: u32,
}

/// INIT flag bit 30: "flags2 is valid".
pub const INIT_EXT: u32 = 1 << 30;

impl InitIn {
    /// Reads the INIT body (the bytes after the header). Returns `None` if `body` is too short.
    pub fn parse(body: &[u8]) -> Option<InitIn> {
        let flags = u32_le_at(body, 12)?;

        // Older kernels send a 16-byte body with no flags2.
        let flags2 = if flags & INIT_EXT != 0 {
            u32_le_at(body, 16).unwrap_or(0)
        } else {
            0
        };

        Some(InitIn {
            major: u32_le_at(body, 0)?,
            minor: u32_le_at(body, 4)?,
            max_readahead: u32_le_at(body, 8)?,
            flags,
            flags2,
        })
    }
}

/// Size of `fuse_out_header`, which starts every reply to the kernel.
pub const OUT_HEADER_LEN: usize = 16;

/// The header at the start of every reply (`struct fuse_out_header` in fuse.h).
#[derive(Debug, PartialEq, Eq)]
pub struct OutHeader {
    pub len: u32,
    /// 0 on success, or a negative errno (e.g. -2 for ENOENT).
    pub error: i32,
    pub unique: u64,
}

impl OutHeader {
    /// Encodes the header as little-endian bytes.
    pub fn to_bytes(&self) -> [u8; OUT_HEADER_LEN] {
        let mut bytes = [0u8; OUT_HEADER_LEN];
        bytes[0..4].copy_from_slice(&self.len.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.error.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.unique.to_le_bytes());
        bytes
    }
}

/// The body of an INIT reply (`struct fuse_init_out` in fuse.h).
#[derive(Debug, PartialEq, Eq)]
pub struct InitOut {
    pub major: u32,
    pub minor: u32,
    pub max_readahead: u32,
    pub flags: u32,
    /// Most background requests (e.g. readahead) in flight at once. 0 = kernel default.
    pub max_background: u16,
    /// Background requests in flight before the kernel slows writers down. 0 = kernel default.
    pub congestion_threshold: u16,
    /// Largest WRITE request we accept, in bytes.
    pub max_write: u32,
    /// Timestamp precision in nanoseconds. 0 or 1 = full precision.
    pub time_gran: u32,
    /// Most pages in one request. Only used with the MAX_PAGES flag.
    pub max_pages: u16,
    /// Only used by DAX (virtio-fs).
    pub map_alignment: u16,
    pub flags2: u32,
    /// Only used with the PASSTHROUGH flag.
    pub max_stack_depth: u32,
    /// Only used with the REQUEST_TIMEOUT flag.
    pub request_timeout: u16,
}

/// Size of `fuse_init_out`.
pub const INIT_OUT_LEN: usize = 64;

impl InitOut {
    /// Encodes the body as little-endian bytes. The unused tail (bytes 42–63) stays zero.
    pub fn to_bytes(&self) -> [u8; INIT_OUT_LEN] {
        let mut bytes = [0u8; INIT_OUT_LEN];
        bytes[0..4].copy_from_slice(&self.major.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.minor.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.max_readahead.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        bytes[16..18].copy_from_slice(&self.max_background.to_le_bytes());
        bytes[18..20].copy_from_slice(&self.congestion_threshold.to_le_bytes());
        bytes[20..24].copy_from_slice(&self.max_write.to_le_bytes());
        bytes[24..28].copy_from_slice(&self.time_gran.to_le_bytes());
        bytes[28..30].copy_from_slice(&self.max_pages.to_le_bytes());
        bytes[30..32].copy_from_slice(&self.map_alignment.to_le_bytes());
        bytes[32..36].copy_from_slice(&self.flags2.to_le_bytes());
        bytes[36..40].copy_from_slice(&self.max_stack_depth.to_le_bytes());
        bytes[40..42].copy_from_slice(&self.request_timeout.to_le_bytes());
        bytes
    }
}

/// Protocol version rufuse speaks. The struct layouts in this file follow 7.45.
pub const MAJOR_VERSION: u32 = 7;
pub const MINOR_VERSION: u32 = 45;

/// Largest WRITE we accept, until it becomes configurable. Also the kernel's limit without MAX_PAGES.
pub const DEFAULT_MAX_WRITE: u32 = 128 * 1024;

/// Builds the full reply (header + body) to the INIT request `unique`. No optional features yet.
pub fn init_reply(unique: u64, init: &InitIn) -> Vec<u8> {
    let body = InitOut {
        major: MAJOR_VERSION,
        minor: MINOR_VERSION,
        max_readahead: init.max_readahead,
        flags: 0,
        max_background: 0,
        congestion_threshold: 0,
        max_write: DEFAULT_MAX_WRITE,
        time_gran: 1,
        max_pages: 0,
        map_alignment: 0,
        flags2: 0,
        max_stack_depth: 0,
        request_timeout: 0,
    };
    let len = OUT_HEADER_LEN + INIT_OUT_LEN;
    let header = OutHeader { len: len as u32, error: 0, unique };

    let mut reply = Vec::with_capacity(len);
    reply.extend_from_slice(&header.to_bytes());
    reply.extend_from_slice(&body.to_bytes());
    reply
}

/// Reads a little-endian u32 at `offset`, or `None` if out of bounds.
fn u32_le_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let chunk = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(chunk.try_into().ok()?))
}

/// Reads a little-endian u64 at `offset`, or `None` if out of bounds.
fn u64_le_at(bytes: &[u8], offset: usize) -> Option<u64> {
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

    #[test]
    fn parses_init_body() {
        let body = InitIn::parse(&INIT_REQUEST[IN_HEADER_LEN..]).unwrap();
        assert_eq!(
            body,
            InitIn { major: 7, minor: 45, max_readahead: 0x20000, flags: 0x73fffffb, flags2: 0x5ff }
        );
    }

    #[test]
    fn ignores_flags2_without_init_ext() {
        let mut body = INIT_REQUEST[IN_HEADER_LEN..].to_vec();
        body[15] &= !0x40; // clear bit 30 (INIT_EXT) in flags
        assert_eq!(InitIn::parse(&body).unwrap().flags2, 0);
    }

    #[test]
    fn parses_old_16_byte_init_body() {
        let body = InitIn::parse(&INIT_REQUEST[IN_HEADER_LEN..IN_HEADER_LEN + 16]).unwrap();
        assert_eq!(body.minor, 45);
        assert_eq!(body.flags2, 0);
    }

    #[test]
    fn rejects_short_init_body() {
        assert_eq!(InitIn::parse(&INIT_REQUEST[IN_HEADER_LEN..IN_HEADER_LEN + 15]), None);
    }

    #[test]
    fn encodes_out_header() {
        let header = OutHeader { len: 80, error: -2, unique: 2 };
        assert_eq!(
            header.to_bytes(),
            [
                0x50, 0x00, 0x00, 0x00, 0xfe, 0xff, 0xff, 0xff, // len = 80, error = -2 (ENOENT)
                0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // unique = 2
            ]
        );
    }

    #[test]
    fn encodes_init_out() {
        // Every field gets a different value, so a field written to the wrong offset shows up.
        let body = InitOut {
            major: 7,
            minor: 31,
            max_readahead: 0x20000,
            flags: 0x01020304,
            max_background: 0x0506,
            congestion_threshold: 0x0708,
            max_write: 0x090a0b0c,
            time_gran: 1,
            max_pages: 0x0d0e,
            map_alignment: 0x0f10,
            flags2: 0x11121314,
            max_stack_depth: 2,
            request_timeout: 0x1516,
        };
        let expected: [u8; INIT_OUT_LEN] = [
            0x07, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, // major, minor
            0x00, 0x00, 0x02, 0x00, 0x04, 0x03, 0x02, 0x01, // max_readahead, flags
            0x06, 0x05, 0x08, 0x07, 0x0c, 0x0b, 0x0a, 0x09, // max_background, congestion_threshold, max_write
            0x01, 0x00, 0x00, 0x00, 0x0e, 0x0d, 0x10, 0x0f, // time_gran, max_pages, map_alignment
            0x14, 0x13, 0x12, 0x11, 0x02, 0x00, 0x00, 0x00, // flags2, max_stack_depth
            0x16, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // request_timeout, unused...
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(body.to_bytes(), expected);
    }

    #[test]
    fn builds_init_reply() {
        let init = InitIn::parse(&INIT_REQUEST[IN_HEADER_LEN..]).unwrap();
        let reply = init_reply(2, &init);

        assert_eq!(reply.len(), 80);
        assert_eq!(reply[..OUT_HEADER_LEN], OutHeader { len: 80, error: 0, unique: 2 }.to_bytes());

        let body = &reply[OUT_HEADER_LEN..];
        assert_eq!(u32_le_at(body, 0), Some(MAJOR_VERSION));
        assert_eq!(u32_le_at(body, 4), Some(MINOR_VERSION));
        assert_eq!(u32_le_at(body, 8), Some(0x20000)); // the kernel's max_readahead
        assert_eq!(u32_le_at(body, 12), Some(0)); // no flags
        assert_eq!(u32_le_at(body, 20), Some(DEFAULT_MAX_WRITE));
    }
}
