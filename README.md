# Code Challenge — Gateway control protocol (40 minutes)

## The situation

A client agent runs on users' laptops. It keeps a TCP connection to a gateway and talks to it over a small binary control protocol. This crate turns the bytes that arrive from the gateway into Rust types.

A colleague wrote it quickly for a demo. It works with the well-formed frames the gateway sends today, and its tests pass. Before it ships to customer laptops, it must handle whatever the network delivers. That work is ticket **AGENT-318** below.

You may use your usual tools, including an AI assistant and the documentation at docs.rs. Please share your screen and explain what you ask the assistant and how you check what it gives you.

## The protocol

Every frame has a 9-byte header and a payload. Numbers are big-endian.

| Field | Size | Value |
|---|---|---|
| magic | 2 bytes | `GW` (`0x47 0x57`) |
| version | u8 | `1` today (`2` in stage 3) |
| message type | u8 | `1` = Hello, `2` = Data |
| flags | u8 | any value; kept as is |
| payload length | u32 | number of payload bytes; at most 65,536 |
| payload | `length` bytes | depends on the message type |

Payloads in version 1:

- **Hello** (type 1): exactly 12 bytes. Session id (u64), then the keepalive interval in seconds (u32).
- **Data** (type 2): any bytes, possibly none.

There are no other message types.

## Ticket AGENT-318 — Make the frame parser safe for production

Do the stages in order. Each stage has acceptance criteria; show each one with a test.

**Stage 1 — Never panic on bad input**

1. `parse` returns `Result<Frame, ParseError>`, with your own `ParseError` type.
2. Malformed input returns an error, never a panic: truncated input, wrong magic, unknown message type, payload length above 65,536, and a payload that does not match its message type.
3. Every field is decoded exactly as the protocol table describes. No value is silently changed.
4. A test checks that every prefix of a valid frame (all its first *n* bytes) returns an error or a frame, and never panics.

**Stage 2 — Bytes arrive in chunks**

The socket delivers bytes in chunks of any size: a chunk can hold several frames, or only part of one.

5. A decoder (you choose the name and the API) accepts chunks as they arrive and returns each complete frame once, in order.
6. It works when several frames arrive in one chunk, and when a frame, including its header, is split across chunks at any byte.
7. Memory stays bounded: a frame that announces a payload above 65,536 bytes is rejected without waiting for or storing its payload.

**Stage 3 — Protocol version 2**

The gateway team is adding version 2. In version 2, a Hello payload is the 12 bytes of version 1, followed by zero or more extensions. Each extension is: type (u8), length (u16), value (`length` bytes). Extension type `1` is an MTU hint (u16). Data is the same as in version 1.

8. Version 1 frames still parse, and the version 1 byte format does not change.
9. Version 2 Hello frames parse, with and without extensions. The MTU hint is available when present. Unknown extension types are skipped.
10. Decide what the client does with a version it does not know (for example `3`), test it, and explain why.

**Stage 4 — Discussion (no code)**

11. Be ready to discuss: how you would plug your decoder into a Tokio `TcpStream`; what happens when the code that consumes frames is slower than the network; and what else could make code like this panic in production even when all tests pass.

## Steps

1. **Read (about 3 minutes).** Read this file, `src/lib.rs` and `tests/parse.rs`. Run `cargo test`.
2. **Stage 1 (about 15 minutes).**
3. **Stage 2 (about 12 minutes).**
4. **Stage 3 (about 6 minutes).** If time is short, explain how you would finish it.
5. **Stage 4 (about 4 minutes).** Talk it through with the interviewer.

Not finishing every stage is expected. We are interested in how you reason, not only in the final code.

## Rules

- Keep the existing code as the starting point; do not rewrite it from scratch. You may change `parse` and the types.
- Check the code against the protocol and the ticket; do not assume it is correct because the tests pass.
- The standard library only. Do not add crates.
- Keep the existing tests. You may adapt them to a new signature (for example, add `.unwrap()`), but do not delete or weaken them.
- If you use an AI assistant, review every change it proposes before accepting it, and be ready to explain it.

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

Think out loud as you work.
