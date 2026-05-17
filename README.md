# rs_imessage

> **Unstable — still in development.** APIs and behavior may change without notice.

Library-first iMessage toolkit for macOS: read `chat.db`, stream new messages,
send via Messages.app (AppleScript), optional HTTP bridge and private-api dylib.

## Mac hosts the bridge

```text
┌──────────────────────────── Mac (always on) ────────────────────────────┐
│  Messages.app  →  chat.db  →  rs_imessage bridge (HTTP on :8721)        │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │  LAN / Tailscale / SSH tunnel
                                     ▼
┌──────────────────────────── Your agent host ──────────────────────────────┐
│  Any client  →  RS_IMESSAGE_URL + Bearer token  →  send + SSE events      │
└───────────────────────────────────────────────────────────────────────────┘
```

```bash
export RS_IMESSAGE_TOKEN="$(openssl rand -hex 24)"
cargo run --features cli -- serve --bind 0.0.0.0:8721
```

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Liveness (no auth) |
| `/api/v1/ping` | GET | Auth check |
| `/api/v1/chats` | GET | List chats |
| `/api/v1/messages/history` | POST | History for one chat |
| `/api/v1/messages/send` | POST | Send text / file |
| `/api/v1/events` | GET | SSE stream of new messages |

**Same machine:** use `Client` in-process. **Remote:** set `RS_IMESSAGE_URL` (legacy `RS_IMSG_URL` still read where noted in code).

## Requirements

- macOS 14+
- Messages.app signed in
- **Full Disk Access** for the process using this library
- **Automation** permission for Messages when calling `send`

## Library

```toml
rs_imessage = "0.1"
```

```rust
use rs_imessage::{Client, ClientConfig};
use rs_imessage::watch::WatchOptions;

let client = Client::open(ClientConfig::default())?;
let chats = client.list_chats(20)?;
let mut stream = client.watch(WatchOptions::default())?;
```

### Private API (`private-api` feature)

Build the MIT [openclaw/imsg](https://github.com/openclaw/imsg) helper dylib:

```bash
./scripts/build-bridge-from-imsg.sh
cargo build --features private-api
```

FaceTime Audio lives in [`rs_facetime`](https://github.com/undivisible/rs_facetime).

## CLI

```bash
cargo build --features cli
./target/debug/rs_imessage chats --limit 10 --json
```

`RS_IMESSAGE_DB` overrides `~/Library/Messages/chat.db` (`RS_IMSG_DB` accepted as legacy).

## Related

- [rs_facetime](https://github.com/undivisible/rs_facetime) — FaceTime Audio bridge (separate crate)
