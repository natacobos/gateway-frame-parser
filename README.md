# Code Challenge — Gateway control protocol (70 minutes)

## The situation

A program called the **agent** runs on users' laptops. It keeps a network
connection (TCP) open to a server called the **gateway**, and the two talk
through a small binary protocol: messages are sent as raw bytes, not as text.
This crate reads those bytes and turns them into Rust types.

A colleague wrote it quickly for a demo. It works with the well-formed
messages the gateway sends today, and its 4 tests pass. But before it ships to
customer laptops it must survive **anything** the network can deliver: cut
messages, garbage, or a message that claims to be enormous. That work is ticket
**AGENT-318** below.

## The protocol

Every frame has a 9-byte header and a payload. Numbers are big-endian (most
significant byte first, as `u32::from_be_bytes` reads them).

| Field | Size | Value |
|---|---|---|
| magic | 2 bytes | `GW` (`0x47 0x57`) |
| version | u8 | `1` today (`2` in stage 3) |
| message type | u8 | `1` = Hello, `2` = Data |
| flags | u8 | any value; kept as is |
| payload length | u32 | number of payload bytes; at most 65,536 |
| payload | `length` bytes | depends on the message type |

Payloads in version 1:

- **Hello** (type 1): exactly 12 bytes. Session id (u64), then the keepalive
  interval in seconds (u32): how often the agent must send a sign of life.
- **Data** (type 2): any bytes, possibly none.

There are no other message types.

## Ticket AGENT-318 — Make the frame parser safe for production

Each message is called a **frame**: a 9-byte header followed by its payload.
Do the stages in order. For each one, show the interviewer a test that proves
it works.

### Stage 1 — Never crash on bad input (about 20 minutes)

Today `parse` crashes (panics) when the input is short or malformed, and it
silently accepts some bad input. Change it so that:

1. `parse` returns `Result<Frame, ParseError>`, where `ParseError` is an error
   type you define.
2. Bad input returns an error, never a panic. At least: input that is cut
   short, a wrong magic (the first two bytes are not `GW`), an unknown message
   type, a payload length above 65,536, and a payload that does not match its
   message type (for example a Hello that is not 12 bytes).
3. Every field is read exactly as the protocol table says. No value may be
   silently changed.
4. A test feeds `parse` every prefix of a valid frame (its first 0 bytes, its
   first 1 byte, its first 2 bytes, and so on) and checks it never panics.

### Stage 2 — Bytes arrive in pieces (about 20 minutes)

A network socket does not deliver one frame at a time. Each read gives you a
**chunk** of any size: it can hold several frames, or only part of one, even
part of the header.

5. Write a decoder (you choose the name and how it is used) that receives
   chunks as they arrive and returns each complete frame once, in order.
6. It works when several frames come in one chunk, and when a frame, including
   its header, is split across chunks at any byte. Test both, for example by
   feeding one byte at a time.
7. Memory stays bounded: a frame whose header announces a payload above 65,536
   bytes is rejected as soon as the header arrives, without waiting for or
   storing its payload.

### Stage 3 — Protocol version 2 (about 15 minutes)

The gateway team is adding version 2 of the protocol. In version 2, a Hello
payload is the same 12 bytes as in version 1, followed by zero or more
**extensions**. Each extension is: type (u8), length (u16), value (`length`
bytes). Extension type `1` is an MTU hint (a u16: the largest packet size the
network allows). Data messages do not change.

8. Version 1 frames still parse exactly as before.
9. Version 2 Hello frames parse, with and without extensions. The MTU hint is
   available when present. Extension types you do not know are skipped.
10. Decide what the agent does with a version it does not know (for example
    `3`), test it, and explain why.

If time is short, explain how you would finish this stage.

### Stage 4 — Discussion, no code (about 10 minutes)

The interviewer will ask you a few questions, for example how you would
connect your decoder to a real network connection in Tokio, and what happens
when the code that uses the frames is slower than the network.

## Steps

1. **Read (about 5 minutes).** Read this file, `src/lib.rs` and
   `tests/parse.rs`. Run `cargo test`. Tell the interviewer what you see before
   you change anything.
2. Stages 1 to 4, in order.

Not finishing every stage is expected. We are interested in how you reason,
not only in the final code. Think out loud as you work.

## Rules

- Start from the existing code; do not rewrite it from scratch. You may change
  `parse` and the types.
- Check the code against the protocol table; do not assume it is correct
  because the tests pass.
- Standard library only. Do not add crates.
- Keep the existing tests. You may adapt them to the new signature (for
  example, add `.unwrap()` in the tests), but do not delete or weaken them.
- **No AI tools at any point:** no AI chat, no coding agents and no AI code
  completion in your editor (turn off GitHub Copilot or similar before you
  start). You may use docs.rs, the standard library docs and a regular web
  search (skip the AI-generated answers at the top of the results).

## Running the project

**With Rust installed (recommended).** Use the latest stable Rust (1.85 or newer; install or update with `rustup update stable`). From the project folder:

```bash
cargo test
```

The crate has no dependencies, so it builds offline in about a second.

**With Docker.** If you cannot install Rust, use the official image (about 290–330 MB to download):

```bash
docker run --rm -v "$PWD":/w -w /w rust:1.99.0-slim cargo test
```

On Linux, add `-u "$(id -u):$(id -g)"` after `--rm` so the `target/` folder belongs to your user.

## Files

- `src/lib.rs`: the frame types and `parse`.
- `tests/parse.rs`: the existing tests, with a helper that builds frames.
- `Cargo.toml`: the crate (edition 2021, no dependencies).
