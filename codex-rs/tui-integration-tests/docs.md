# Noridoc: TUI Integration Tests

Path: @/codex-rs/tui-integration-tests

### Overview

- Black-box integration testing framework for the Codex TUI using PTY (pseudo-terminal) emulation
- Spawns the real `codex` binary in a simulated terminal and exercises full application stack
- Uses VT100 parser to capture and validate terminal screen output via snapshot testing
- Provides programmatic keyboard input simulation and screen state polling

### How it fits into the larger codebase

- Tests the complete integration between `@/codex-rs/cli`, `@/codex-rs/tui`, `@/codex-rs/core`, and `@/codex-rs/acp`
- Complements unit tests in `@/codex-rs/tui/src/chatwidget.rs` by testing full application behavior
- Uses `@/codex-rs/mock-acp-agent` as the ACP backend for deterministic test scenarios
- Validates CLI argument parsing, TUI event loop, ACP protocol communication, and terminal rendering
- Part of the workspace at `@/codex-rs/Cargo.toml:46`

### Core Implementation

**Test Harness:** `TuiSession` in `@/codex-rs/tui-integration-tests/src/lib.rs`

The main API provides:
- `spawn(rows, cols)` - Launch codex binary with mock-acp-agent in PTY with automatic temp directory
- `spawn_with_config(rows, cols, config)` - Launch with custom configuration and automatic temp directory
- `send_str(text)` - Simulate typing text
- `send_key(key)` - Send keyboard events (Enter, Escape, Ctrl-C, etc.)
- `wait_for_text(needle, timeout)` - Poll screen until text appears
- `wait_for(predicate, timeout)` - Poll screen until condition matches
- `screen_contents()` - Get current terminal screen as string

**Automatic Test Isolation:**

All tests run in isolated temporary directories created in `/tmp/`:
- Each `spawn()` or `spawn_with_config()` call creates a new temp directory
- Directory contains a `hello.py` file with `print('Hello, World!')`
- Temp directory is automatically cleaned up when `TuiSession` is dropped
- Tests no longer run in user's home directory for better isolation

**Architecture:**

```
Test Code
    ↓
TuiSession (portable_pty)
    ↓
PTY Master ←→ PTY Slave
    ↓           ↓
VT100 Parser   codex binary (--model mock-acp-agent)
    ↓           ↓
Screen State   ACP JSON-RPC over stdin/stdout
                ↓
            mock_acp_agent (env var configured)
```

**Key Input Handling:** `Key` enum in `@/codex-rs/tui-integration-tests/src/keys.rs`

Converts high-level key events to ANSI escape sequences:
- `Key::Enter` → `\r`
- `Key::Escape` → `\x1b`
- `Key::Up/Down/Left/Right` → `\x1b[A/B/D/C`
- `Key::Backspace` → `\x7f`
- `Key::Ctrl('c')` → Control character encoding

**Session Configuration:** `SessionConfig` in `@/codex-rs/tui-integration-tests/src/lib.rs`

Builder pattern for test environment setup:
- `with_mock_response(text)` - Set `MOCK_AGENT_RESPONSE` env var
- `with_stream_until_cancel()` - Set `MOCK_AGENT_STREAM_UNTIL_CANCEL=1`
- `with_agent_env(key, value)` - Pass custom env vars to mock agent
- `cwd` field - Optional working directory (auto-created temp directory if None)

### Things to Know

**Test Files Structure:**

| File | Coverage |
|------|----------|
| `@/codex-rs/tui-integration-tests/tests/startup.rs` | TUI initialization and prompt display |
| `@/codex-rs/tui-integration-tests/tests/prompt_flow.rs` | Prompt submission and agent responses |
| `@/codex-rs/tui-integration-tests/tests/input_handling.rs` | Text editing, backspace, Ctrl-C clearing |
| `@/codex-rs/tui-integration-tests/tests/cancellation.rs` | Stream cancellation with Escape key |

**Snapshot Testing with Insta:**

Tests use `insta::assert_snapshot!()` to capture terminal output:
```rust
assert_snapshot!("startup_screen", session.screen_contents());
```

Snapshots stored in `@/codex-rs/tui-integration-tests/snapshots/*.snap` for regression detection.

**PTY Implementation Details:**

- Uses `portable-pty` crate for cross-platform PTY support
- Sets `TERM=xterm-256color` for terminal feature detection
- NO_COLOR=1 by default for deterministic output parsing
- Terminal size configurable (default 24x80, some tests use 40x120)

**Polling Pattern:**

`poll()` method attempts non-blocking read from PTY master:
- Reads up to 8KB buffer per poll
- Intercepts and responds to terminal control sequences before parsing
- Feeds processed data to VT100 parser incrementally
- Returns immediately on WouldBlock (no data available)
- `wait_for()` loops with 50ms sleep between polls

**Control Sequence Interception:**

The `intercept_control_sequences()` method handles terminal queries that require responses:
- Detects cursor position query (`ESC[6n`) in output stream from codex binary
- Writes cursor position response (`ESC[1;1R`) back to PTY input
- Removes control sequences from parser stream to avoid rendering artifacts
- Enables crossterm terminal initialization without real terminal support

**Mock Agent Integration:**

Tests control mock agent behavior via environment variables:
- `MOCK_AGENT_RESPONSE` - Custom response text instead of defaults
- `MOCK_AGENT_DELAY_MS` - Simulate streaming delays
- `MOCK_AGENT_STREAM_UNTIL_CANCEL` - Stream until Escape pressed

See `@/codex-rs/mock-acp-agent/docs.md` for full list of env vars.

**Binary Discovery:**

`codex_binary_path()` locates the compiled binary:
```
test_exe: target/debug/deps/startup-abc123
          ↓
target/debug/deps (parent)
          ↓
target/debug (parent.parent)
          ↓
target/debug/codex (join "codex")
```

**Known Limitations:**

- VT100 parser may not perfectly emulate all terminal behaviors
- Terminal size changes after spawn not currently supported
- Color codes disabled (NO_COLOR=1) for test determinism

**Dependencies:**

- `portable-pty = "0.8"` - PTY creation and management
- `vt100 = "0.15"` - Terminal emulator/parser
- `insta = "1"` - Snapshot testing framework
- `anyhow = "1"` - Error handling
- `tempfile = "3"` - Temporary directory creation for test isolation

Created and maintained by Nori.
