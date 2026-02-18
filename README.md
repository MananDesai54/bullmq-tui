# BullMQ TUI (Just Use TUI as of Now, Web and App is not ready yet)

A cross-platform terminal user interface for monitoring BullMQ queues, built in Rust.

![Terminal Demo](docs/terminal-demo.png)

## Features

- **Queue Discovery**: Auto-detect BullMQ queues via SCAN (production-safe, never uses KEYS)
- **Status Tabs**: View jobs by state - ACTIVE, WAITING, COMPLETED, FAILED, DELAYED, PRIORITIZED, WAITING_CHILDREN
- **Job Details**: Inspect job data, progress, timestamps, attempts, errors, and return values
- **Real-time Updates**: Subscribe to Redis Streams for live job events
- **Job Actions**: Retry failed jobs, remove jobs from any state
- **Queue Actions**: Pause and resume queue processing
- **Cross-Platform**: Runs in terminal, web browser, and as a native desktop app

## Platforms

| Platform | Technology | Redis Connection |
|----------|------------|------------------|
| Terminal | Crossterm + Ratatui | Direct |
| Web | Ratzilla (WASM) | Via WebSocket Proxy |
| Desktop | Tauri | Direct |

## Quick Start

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Redis server with BullMQ queues
- For web: [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- For desktop: [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Terminal App

The fastest way to get started:

```bash
# Clone and build
git clone https://github.com/centerseat/bullmq-tui.git
cd bullmq-tui
cargo build --release -p bullmq-tui-terminal

# Run
./target/release/bullmq-tui --redis redis://localhost:6379
```

### Web App

Requires running the WebSocket proxy since browsers cannot connect directly to Redis.

```bash
# Terminal 1: Start the proxy
cargo run -p bullmq-proxy

# Terminal 2: Build and serve the web app
cd crates/bullmq-tui-web
trunk serve
```

Open http://localhost:8080 in your browser.

### Desktop App

Native desktop application using Tauri. Connects directly to Redis (no proxy needed).

```bash
cd crates/bullmq-tui-desktop

# Install dependencies
npm install

# Development mode
npm run dev

# Build for production (creates installer)
npm run build
```

## Usage

### Command Line Options

```
bullmq-tui [OPTIONS]

Options:
  -r, --redis <URL>     Redis connection URL [default: redis://localhost:6379]
  -i, --interval <SEC>  Refresh interval in seconds [default: 5]
  -d, --debug           Enable debug logging
  -h, --help            Print help
  -V, --version         Print version
```

### Examples

```bash
# Connect to local Redis
bullmq-tui

# Connect to remote Redis with password
bullmq-tui --redis redis://:password@redis.example.com:6379

# Connect to Redis cluster
bullmq-tui --redis redis://redis-cluster.example.com:6379

# Enable debug logging
bullmq-tui --debug

# Faster refresh rate
bullmq-tui --interval 1
```

## Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `Ctrl+C` | Quit application |
| `?` | Show help screen |

### Queue List View

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Open selected queue |
| `r` | Refresh queue list |
| `q` | Quit application |

### Job List View

| Key | Action |
|-----|--------|
| `Tab` | Next status tab |
| `Shift+Tab` | Previous status tab |
| `1-7` | Jump to specific tab (1=Active, 2=Waiting, etc.) |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | View job details |
| `r` | Retry selected job (if failed) |
| `d` | Delete selected job |
| `p` | Pause/Resume queue |
| `Ctrl+D` / `PageDown` | Page down |
| `Ctrl+U` / `PageUp` | Page up |
| `R` | Refresh job list |
| `q` | Back to queue list |

### Job Detail View

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `r` | Retry job (if failed) |
| `d` | Delete job |
| `q` | Back to job list |

### Dialogs

| Key | Action |
|-----|--------|
| `y` | Confirm action |
| `n` / `Esc` | Cancel action |

## Architecture

```
┌─────────────────────────────────────┐
│          bullmq-tui-core            │
│   (Shared TEA: Model/Update/View)   │
│         Platform-agnostic UI        │
└──────────────────┬──────────────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
    ▼              ▼              ▼
┌────────┐   ┌──────────┐   ┌──────────┐
│Terminal│   │   Web    │   │ Desktop  │
│Crossterm│   │ Ratzilla │   │  Tauri   │
└────┬───┘   └────┬─────┘   └────┬─────┘
     │            │              │
     │      ┌─────▼─────┐        │
     │      │ WS Proxy  │        │
     │      │  (Axum)   │        │
     │      └─────┬─────┘        │
     └────────────┼──────────────┘
                  ▼
          ┌─────────────┐
          │ bullmq-core │
          │ (Redis Ops) │
          └──────┬──────┘
                 ▼
          ┌─────────────┐
          │   Redis     │
          │  (BullMQ)   │
          └─────────────┘
```

### Design Principles

1. **Elm Architecture (TEA)**: The UI follows the Model-Update-View pattern for predictable state management
2. **Platform Abstraction**: Core UI logic is shared across all platforms via `bullmq-tui-core`
3. **Production-Safe**: Uses SCAN instead of KEYS for queue discovery, connection pooling for efficiency
4. **Real-time**: Subscribes to Redis Streams for live event updates

## Project Structure

```
bullmq-tui/
├── Cargo.toml                      # Workspace manifest
├── README.md
├── crates/
│   ├── bullmq-core/                # Redis client library
│   │   └── src/
│   │       ├── lib.rs              # Public API exports
│   │       ├── client.rs           # BullMQClient trait definition
│   │       ├── redis_client.rs     # Redis implementation
│   │       ├── queue.rs            # Queue discovery & operations
│   │       ├── job.rs              # Job CRUD operations
│   │       ├── events.rs           # Redis Stream subscription
│   │       ├── error.rs            # Error types
│   │       └── models/             # Data models
│   │           ├── job.rs          # Job, JobState, JobOptions
│   │           ├── queue.rs        # QueueInfo, QueueCounts
│   │           └── event.rs        # QueueEvent, EventType
│   │
│   ├── bullmq-tui-core/            # Shared UI logic
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── model.rs            # Application state
│   │       ├── message.rs          # Action messages
│   │       ├── update.rs           # State transitions
│   │       ├── effects.rs          # Side effects
│   │       └── view/               # Ratatui components
│   │           ├── mod.rs          # Main render function
│   │           ├── queue_list.rs   # Queue list view
│   │           ├── job_table.rs    # Job table view
│   │           ├── job_detail.rs   # Job detail view
│   │           ├── status_tabs.rs  # Status tab bar
│   │           ├── help.rs         # Help overlay
│   │           └── widgets.rs      # Reusable widgets
│   │
│   ├── bullmq-tui-terminal/        # Native terminal app
│   │   └── src/
│   │       ├── main.rs             # Entry point
│   │       └── event_handler.rs    # Keyboard handling
│   │
│   ├── bullmq-tui-web/             # WASM web app
│   │   ├── index.html              # HTML shell
│   │   ├── Trunk.toml              # Build config
│   │   └── src/
│   │       ├── lib.rs              # WASM entry point
│   │       └── ws_client.rs        # WebSocket client
│   │
│   └── bullmq-tui-desktop/         # Tauri desktop app
│       ├── package.json
│       ├── src/
│       │   └── index.html          # Desktop UI
│       └── src-tauri/
│           ├── Cargo.toml
│           ├── tauri.conf.json
│           └── src/
│               ├── main.rs         # Tauri entry
│               └── commands.rs     # IPC commands
│
└── proxy/                          # WebSocket-to-Redis proxy
    ├── Cargo.toml
    └── src/
        └── main.rs                 # Axum WebSocket server
```

## BullMQ Redis Key Patterns

The TUI interacts with the following Redis keys (read-only except for job actions):

```
bull:{queue}:meta           # Queue metadata (hash)
bull:{queue}:id             # Job ID counter (string)
bull:{queue}:wait           # Waiting jobs (list)
bull:{queue}:active         # Active jobs (list)
bull:{queue}:delayed        # Delayed jobs (sorted set, score=timestamp)
bull:{queue}:completed      # Completed jobs (sorted set)
bull:{queue}:failed         # Failed jobs (sorted set)
bull:{queue}:prioritized    # Prioritized jobs (sorted set)
bull:{queue}:waiting-children  # Jobs waiting for children (sorted set)
bull:{queue}:paused         # Pause marker (string)
bull:{queue}:{jobId}        # Job data (hash)
bull:{queue}:events         # Event stream (stream)
```

## Development

### Building

```bash
# Build all crates
cargo build --workspace

# Build release
cargo build --release --workspace

# Build specific crate
cargo build -p bullmq-tui-terminal
```

### Running in Development

```bash
# Terminal app with debug logging
cargo run -p bullmq-tui-terminal -- --debug

# Proxy with debug logging
RUST_LOG=debug cargo run -p bullmq-proxy

# Web app with hot reload
cd crates/bullmq-tui-web && trunk serve

# Desktop app
cd crates/bullmq-tui-desktop && npm run dev
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Test specific crate
cargo test -p bullmq-core
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all --check

# Run clippy
cargo clippy --workspace -- -D warnings

# Check for unused dependencies
cargo +nightly udeps --workspace
```

## Environment Variables

### Terminal App

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | `warn` |

### Proxy

| Variable | Description | Default |
|----------|-------------|---------|
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `PORT` | WebSocket server port | `3001` |
| `RUST_LOG` | Log level | `info` |

## Troubleshooting

### Connection Issues

**"Connection refused"**
- Ensure Redis is running: `redis-cli ping`
- Check the Redis URL is correct
- Verify network connectivity

**"Authentication failed"**
- Include password in URL: `redis://:password@host:port`
- Check Redis ACL permissions

### No Queues Found

- BullMQ queues are discovered via `bull:*:meta` and `bull:*:id` keys
- Ensure your application has created at least one queue
- Check Redis database number (default is 0)

### Web App Not Connecting

- Ensure the proxy is running on port 3001
- Check browser console for WebSocket errors
- Verify CORS is not blocking the connection

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Run lints: `cargo clippy --workspace`
6. Commit: `git commit -am 'Add my feature'`
7. Push: `git push origin feature/my-feature`
8. Create a Pull Request

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Redis Client | redis-rs + deadpool-redis |
| TUI Framework | Ratatui |
| Terminal Backend | Crossterm |
| Web Backend | Ratzilla (WASM) |
| Desktop | Tauri v2 |
| WebSocket Proxy | Axum |
| Async Runtime | Tokio |

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [BullMQ](https://github.com/taskforcesh/bullmq) - The queue system this monitors
- [Ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [Ratzilla](https://github.com/ratzilla/ratzilla) - WASM backend for Ratatui
- [Tauri](https://tauri.app/) - Desktop app framework
