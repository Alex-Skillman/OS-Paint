# OS-Paint

A real-time collaborative whiteboard. Draw on a shared canvas with friends over a peer-to-peer connection — no accounts, no installs beyond the app itself, just a room code.

Built in Rust with [macroquad](https://github.com/not-fl3/macroquad) for rendering/input and [matchbox](https://github.com/johanhelsing/matchbox) for WebRTC peer-to-peer networking.

## Features

- **Shared canvas** — a fixed-size (3200×2400) drawing surface everyone in a room sees identically, panned and zoomed independently per viewer.
- **Tools** — pen, point eraser, and stroke eraser (removes whole strokes at once), each with an adjustable size.
- **Color picker** — a quick-pick palette (defaults plus your most recently used colors) and a full hue/saturation color wheel.
- **Undo/redo** — up to 50 steps, synced to peers so everyone converges on the same canvas.
- **Live peer cursors** — see where everyone else in the room is pointing and what color they're about to draw with.
- **Lobbies** — host a room to get a random 6-digit code, or join one someone shares with you. No sign-up required.

## Controls

| Action | Input |
|---|---|
| Draw | Left-click drag (Pen tool) |
| Erase | Left-click drag (Eraser / Stroke Eraser tool) |
| Pan | Right-click drag |
| Zoom | Mouse wheel |
| Change tool size | Up / Down arrow keys |
| Switch to Pen | `D` |
| Switch to Eraser | `E` |
| Switch to Stroke Eraser | `R` |
| Undo | Ctrl+Z |
| Redo | Ctrl+Y |
| Open menu (host/join/leave lobby) | Esc |

## Running from source

Requires a [Rust toolchain](https://rustup.rs/).

```bash
cargo run --release
```

By default the app connects to a public signaling server for matchmaking. To point it at your own instead, set `OS_PAINT_SIGNALING_SERVER` before launching:

```bash
OS_PAINT_SIGNALING_SERVER=wss://your-signaling-server.example {your binary}
```

## Prebuilt binaries

Every push builds Windows, macOS, and Linux binaries via GitHub Actions (see [.github/workflows/build.yml](.github/workflows/build.yml)); grab them from that workflow's run artifacts, or from the [latest release](../../releases/tag/latest), which is republished automatically on every push to `master`.

## Project layout

| File | Responsibility |
|---|---|
| [src/main.rs](src/main.rs) | App loop: state, input dispatch, undo/redo, lobby lifecycle |
| [src/canvas.rs](src/canvas.rs) | Canvas dimensions, pan/zoom clamping, screen↔canvas coordinate mapping |
| [src/stroke.rs](src/stroke.rs) | `Stroke` and `Tool` types |
| [src/input.rs](src/input.rs) | Mouse/keyboard handling for drawing, erasing, panning, zooming |
| [src/render.rs](src/render.rs) | Drawing strokes, the canvas border, and peer cursors to screen |
| [src/ui.rs](src/ui.rs) | Toolbar, size slider, color wheel, and their icons |
| [src/menu.rs](src/menu.rs) | Pause menu: host/join/leave a lobby |
| [src/network.rs](src/network.rs) | Packet types and WebRTC peer sync (strokes, erases, snapshots, cursors) |

## How networking works

Rooms are just everyone who connects to the signaling server with the same 6-digit code — there's no dedicated game server holding state. Each peer:

- Broadcasts individual draw/erase actions to connected peers as they happen.
- Sends a full canvas snapshot to any newly-joined peer, and periodically (every 30s) to all peers, so late joiners and anyone who missed a packet converge on the same canvas.
- Resolves conflicting snapshots by revision number, always keeping the newer one.
