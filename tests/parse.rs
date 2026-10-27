use agent_wire::{parse, Frame, Message, TYPE_DATA, TYPE_HELLO};

/// Builds a frame: magic, version, type, flags, payload length (u32, big-endian), payload.
fn frame(version: u8, msg_type: u8, flags: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = b"GW".to_vec();
    out.extend_from_slice(&[version, msg_type, flags]);
    out.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn hello_payload(session_id: u64, keepalive_secs: u32) -> Vec<u8> {
    let mut p = session_id.to_be_bytes().to_vec();
    p.extend_from_slice(&keepalive_secs.to_be_bytes());
    p
}

#[test]
fn parses_hello() {
    let bytes = frame(1, TYPE_HELLO, 0, &hello_payload(0x0102_0304_0506_0708, 30));
    assert_eq!(
        parse(&bytes),
        Frame {
            version: 1,
            flags: 0,
            message: Message::Hello { session_id: 0x0102_0304_0506_0708, keepalive_secs: 30 },
        }
    );
}

#[test]
fn parses_data() {
    let bytes = frame(1, TYPE_DATA, 0, b"ping");
    assert_eq!(parse(&bytes).message, Message::Data(b"ping".to_vec()));
}

#[test]
fn parses_empty_data() {
    let bytes = frame(1, TYPE_DATA, 0, b"");
    assert_eq!(parse(&bytes).message, Message::Data(Vec::new()));
}

#[test]
fn keeps_version_and_flags() {
    let bytes = frame(1, TYPE_DATA, 0b1010_0001, b"x");
    let parsed = parse(&bytes);
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.flags, 0b1010_0001);
}
