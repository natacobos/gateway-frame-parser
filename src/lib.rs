//! Frames of the gateway control protocol. The format is described in README.md.

pub const MAGIC: [u8; 2] = *b"GW";
pub const HEADER_LEN: usize = 9;

pub const TYPE_HELLO: u8 = 1;
pub const TYPE_DATA: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub version: u8,
    pub flags: u8,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Hello { session_id: u64, keepalive_secs: u16 },
    Data(Vec<u8>),
}

/// Parses the frame at the start of `buf`.
pub fn parse(buf: &[u8]) -> Frame {
    assert!(buf[0..2] == MAGIC, "bad magic");
    let version = buf[2];
    let msg_type = buf[3];
    let flags = buf[4];
    let len = u32::from_be_bytes(buf[5..9].try_into().unwrap());

    let end = (HEADER_LEN as u32 + len) as usize;
    let mut payload = vec![0u8; len as usize];
    payload.copy_from_slice(&buf[HEADER_LEN..end]);

    let message = match msg_type {
        TYPE_HELLO => Message::Hello {
            session_id: u64::from_be_bytes(payload[0..8].try_into().unwrap()),
            keepalive_secs: u32::from_be_bytes(payload[8..12].try_into().unwrap()) as u16,
        },
        _ => Message::Data(payload),
    };

    Frame { version, flags, message }
}
