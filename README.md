# rs_imsg

Library-first iMessage toolkit for macOS — read `chat.db`, stream new messages,
and send via Messages.app (AppleScript). Built for agent runtimes; ships an
optional CLI behind the `cli` feature.

**License:** [Mozilla Public License 2.0](LICENSE)

This crate is **original work**. It is not a fork of the projects listed under
[Acknowledgements](#acknowledgements); those repositories informed design and
API shape only.

## Requirements

- macOS 14+
- Messages.app signed in
- **Full Disk Access** for the process using this library
- **Automation** permission for Messages when calling `send`

## Library

```toml
rs_imsg = { git = "https://github.com/undivisible/rs_imsg", default-features = false }
```

```rust
use rs_imsg::{Client, ClientConfig};
use rs_imsg::watch::WatchOptions;

let client = Client::open(ClientConfig::default())?;
let chats = client.list_chats(20)?;
let mut stream = client.watch(WatchOptions::default())?;
```

On non-macOS targets the crate compiles; macOS-only operations return
`RsImsgError::UnsupportedPlatform`.

### Modules

| Module | Role |
|--------|------|
| `client` | High-level `Client` / `ClientConfig` |
| `db` | Read-only `chat.db` access |
| `watch` | Filesystem notify + poll fallback |
| `send` | AppleScript send path |
| `rpc` | JSON-RPC 2.0 (`run_stdio`) |
| `types` | Stable JSON records |

## Optional CLI

```bash
cargo build --features cli
./target/debug/rs_imsg chats --limit 10 --json
```

Environment: `RS_IMSG_DB` overrides `~/Library/Messages/chat.db`.

## Acknowledgements

We are grateful to the authors of these projects for publishing their work and
shaping the ecosystem. **rs_imsg does not include their source code**; the table
describes conceptual debt only.

| Project | Authors / org | License (as published) | What we learned |
|---------|---------------|------------------------|-----------------|
| [openclaw/imsg](https://github.com/openclaw/imsg) | OpenClaw contributors | MIT | Agent-oriented JSON lines, JSON-RPC over stdio, `watch` with fs events + poll fallback, stderr for human logs |
| [jesec/imessage-rs](https://github.com/jesec/imessage-rs) | Jesse Chan | MIT | Modular crate layout, BlueBubbles-shaped HTTP ideas, group participants, attachment metadata |
| [photon-hq/imessage-kit](https://github.com/photon-hq/imessage-kit) | Photon | MIT | Typed chat/message models, send vs observe semantics, staged attachments |
| [BlueBubblesApp/bluebubbles-server](https://github.com/BlueBubblesApp/bluebubbles-server) | BlueBubbles | GPL-3.0 (server) | REST envelope patterns and route naming for future compatibility |
| [OpenBubbles/openbubbles-app](https://github.com/OpenBubbles/openbubbles-app) | OpenBubbles | Apache-2.0 | Product-level feature set reference (groups, FaceTime, Find My) |

Apple, iMessage, Messages, and FaceTime are trademarks of Apple Inc. This
project is not affiliated with Apple.

## Related repos

- **[mono](https://github.com/atechnology-company/mono)** — hosts `unthinkclaw` and the live gateway; depends on this crate for the macOS iMessage channel.
- **unthinkclaw-live** — archived; content moved into `mono`.

## Roadmap

- Private API feature (typing, edit/unsend, rich send), behind explicit opt-in
- Optional `rs_imsg_http` — BlueBubbles-compatible local server
