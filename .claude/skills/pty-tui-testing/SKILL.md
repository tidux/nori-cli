---
name: pty-tui-testing
description: Use this skill to build features that require testing the TUI with the PTY-based integration test harness.
---

<required>
*CRITICAL* Add the following steps to your Todo list using TodoWrite:

<system-reminder>From this point on, ignore any existing tests until you have a working example validated through a new PTY-based integration test.</system-reminder>
1. Identify the TUI behavior to test (startup, input handling, agent responses, screen updates, etc.)
2. Write a new Rust integration test in `codex-rs/tui-integration-tests/tests/`
   - Use `TuiSession::spawn()` or `TuiSession::spawn_with_config()` to launch codex in a PTY
   - Configure terminal dimensions (rows, cols) appropriate for the test
   - Configure `SessionConfig` with mock agent behavior if needed
3. Follow these steps in a loop until the test passes:
   - Add debug logging using `DEBUG_TUI_PTY=1` environment variable
   - Run the specific test: `cargo test test_name -- --nocapture`
   - Examine the PTY polling behavior, screen contents, and timing
   - Update the test expectations or fix the TUI code
<system-reminder>If you get stuck: did you add DEBUG_TUI_PTY=1 logging?</system-reminder>
4. Review snapshots if using `insta::assert_snapshot!()` and accept with `cargo insta review`
5. Run all TUI integration tests to ensure nothing broke: `cargo test -p tui-integration-tests`
</required>

# PTY-Based TUI Integration Testing

To test the Codex terminal user interface, write Rust integration tests using the `tui-integration-tests` harness. This framework spawns the real `codex` binary in a pseudo-terminal (PTY) and validates terminal output through screen content assertions.

## Core Workflow

**Test Structure:**

All tests follow this pattern:
1. Spawn a TUI session in a PTY with configured dimensions
2. Wait for expected screen content to appear
3. Send keyboard input to simulate user interactions
4. Poll and validate screen state changes
5. Optionally capture snapshots for regression testing

**TUI Session Lifecycle:**

```rust
use tui_integration_tests::{TuiSession, SessionConfig, Key};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn test_tui_behavior() {
    // Spawn codex in a 24x80 terminal with default config
    let mut session = TuiSession::spawn(24, 80)
        .expect("Failed to spawn codex");

    // Wait for welcome message to appear
    session.wait_for_text("To get started", TIMEOUT)
        .expect("Welcome message did not appear");

    // Simulate user typing
    session.send_str("Hello").unwrap();

    // Submit with Enter key
    session.send_key(Key::Enter).unwrap();

    // Wait for agent response
    session.wait_for_text("Test message", TIMEOUT)
        .expect("Agent response did not appear");

    // Assert final screen state
    let contents = session.screen_contents();
    assert!(contents.contains("expected text"));
}
```

**Session Configuration:**

Use `SessionConfig` to control test environment:

```rust
use tui_integration_tests::{TuiSession, SessionConfig, ApprovalPolicy};

let config = SessionConfig::new()
    .with_mock_response("Custom agent response")
    .with_approval_policy(ApprovalPolicy::Never)
    .with_agent_env("MOCK_AGENT_DELAY_MS", "100");

let mut session = TuiSession::spawn_with_config(40, 120, config)
    .expect("Failed to spawn codex");
```

## Key Testing Patterns

**Pattern 1: Startup and Initialization**

Test that the TUI displays correct welcome screens and skips onboarding appropriately:

```rust
#[test]
fn test_startup_shows_welcome() {
    let mut session = TuiSession::spawn_with_config(
        24, 80,
        SessionConfig::default()
            .without_approval_policy()
            .without_sandbox(),
    ).expect("Failed to spawn codex");

    session.wait_for_text("Welcome", TIMEOUT)
        .expect("Welcome did not appear");

    let contents = session.screen_contents();
    assert!(contents.contains("Welcome to Codex"));
    assert!(contents.contains("/tmp/"));
}
```

**Pattern 2: Input Handling and Screen Updates**

Test keyboard input, character echo, and text editing:

```rust
#[test]
fn test_typing_and_backspace() {
    let mut session = TuiSession::spawn(24, 80).unwrap();
    session.wait_for_text("›", TIMEOUT).unwrap();

    // Type text
    session.send_str("Hello World").unwrap();
    session.wait_for_text("Hello World", TIMEOUT).unwrap();

    // Backspace to remove "World"
    for _ in 0..5 {
        session.send_key(Key::Backspace).unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));

    // Verify deletion
    let contents = session.screen_contents();
    assert!(contents.contains("Hello"));
    assert!(!contents.contains("World"));
}
```

**Pattern 3: Agent Interaction and Streaming**

Test agent responses with custom mock behavior:

```rust
#[test]
fn test_agent_response_streaming() {
    let config = SessionConfig::new()
        .with_mock_response("Response line 1\nResponse line 2");

    let mut session = TuiSession::spawn_with_config(24, 80, config).unwrap();
    session.wait_for_text("›", TIMEOUT).unwrap();

    session.send_str("test prompt").unwrap();
    session.send_key(Key::Enter).unwrap();

    // Wait for both lines to stream in
    session.wait_for_text("Response line 1", TIMEOUT).unwrap();
    session.wait_for_text("Response line 2", TIMEOUT).unwrap();
}
```

**Pattern 4: Cancellation and Control Flow**

Test Escape key cancellation and Ctrl-C behavior:

```rust
#[test]
fn test_cancel_streaming_with_escape() {
    let config = SessionConfig::new()
        .with_stream_until_cancel();

    let mut session = TuiSession::spawn_with_config(24, 80, config).unwrap();
    session.wait_for_text("›", TIMEOUT).unwrap();

    session.send_str("test").unwrap();
    session.send_key(Key::Enter).unwrap();

    // Wait for streaming to start
    session.wait_for_text("streaming", TIMEOUT).unwrap();

    // Cancel with Escape
    session.send_key(Key::Escape).unwrap();

    // Verify cancellation message appears
    session.wait_for_text("Cancelled", TIMEOUT).unwrap();
}
```

**Pattern 5: Snapshot Testing**

Capture and validate complete screen state:

```rust
use insta::assert_snapshot;

#[test]
fn test_screen_layout() {
    let mut session = TuiSession::spawn(40, 120).unwrap();
    session.wait_for_text("›", TIMEOUT).unwrap();

    session.send_str("test prompt").unwrap();
    session.send_key(Key::Enter).unwrap();
    session.wait_for_text("Test message", TIMEOUT).unwrap();

    // Capture full screen state for regression testing
    assert_snapshot!("prompt_submitted", session.screen_contents());
}
```

Review snapshots with `cargo insta review` after first run.

## Terminal Dimensions

Choose appropriate terminal size for each test:

- **24x80**: Standard terminal, good for basic prompt flow
- **40x120**: Larger terminal, good for testing layout with more content
- **Small sizes (e.g., 10x40)**: Edge case testing for wrapping behavior

## Configuration Options

**SessionConfig Methods:**

| Method | Purpose |
|--------|---------|
| `with_mock_response(text)` | Set custom agent response instead of defaults |
| `with_stream_until_cancel()` | Make agent stream continuously until Escape pressed |
| `with_agent_env(key, val)` | Pass environment variables to mock agent |
| `with_approval_policy(policy)` | Control approval prompts (Untrusted, OnFailure, OnRequest, Never) |
| `without_approval_policy()` | Remove approval policy to test trust screens |
| `with_sandbox(sandbox)` | Set sandbox level (ReadOnly, WorkspaceWrite, DangerFullAccess) |
| `without_sandbox()` | Remove sandbox to test trust screens |

**ApprovalPolicy Values:**

- `Untrusted`: Only run trusted commands without approval
- `OnFailure`: Ask for approval only when commands fail (default for tests)
- `OnRequest`: Model decides when to ask
- `Never`: Never ask for approval

**Default Test Configuration:**

By default, `SessionConfig::default()` uses:
- `ApprovalPolicy::OnFailure` - Skips initial trust screen
- `Sandbox::WorkspaceWrite` - Allows file operations in tests
- Creates temporary directory in `/tmp/` with `hello.py` file
- Sets `NO_COLOR=1` for deterministic parsing

## TuiSession API

**Spawning:**

- `TuiSession::spawn(rows, cols)` - Launch with defaults in temp directory
- `TuiSession::spawn_with_config(rows, cols, config)` - Launch with custom config

**Input:**

- `send_str(text)` - Simulate typing a string
- `send_key(key)` - Send a keyboard event (Enter, Escape, Backspace, Arrow keys, Ctrl+key)

**Polling and Waiting:**

- `wait_for_text(needle, timeout)` - Poll until text appears on screen
- `wait_for(predicate, timeout)` - Poll until custom condition matches
- `poll()` - Manually read available output and update screen state
- `screen_contents()` - Get current terminal screen as string

**Available Keys:**

- `Key::Enter`, `Key::Escape`, `Key::Backspace`
- `Key::Up`, `Key::Down`, `Key::Left`, `Key::Right`
- `Key::Ctrl('c')`, `Key::Ctrl('d')`, etc.

## Debugging

**Enable Debug Logging:**

```bash
DEBUG_TUI_PTY=1 cargo test test_name -- --nocapture
```

This shows:
- Each `poll()` call and duration
- Read results (bytes read, WouldBlock, EOF)
- `wait_for()` loop iterations and elapsed time
- Screen contents preview at each iteration

**Common Issues:**

1. **Test times out waiting for text**
   - Add `DEBUG_TUI_PTY=1` to see polling behavior
   - Check if text appears but with different formatting/spacing
   - Verify mock agent is configured correctly
   - Increase timeout for slower operations

2. **Snapshot differences**
   - Run `cargo insta review` to inspect changes
   - Check for timing-dependent content (e.g., timestamps)
   - Verify terminal dimensions match snapshot expectations

3. **PTY blocking issues**
   - Poll returns immediately even when no data (non-blocking mode)
   - Use `wait_for()` which polls in a loop with 50ms sleep
   - Don't rely on `poll()` alone for synchronization

4. **Control sequence artifacts**
   - PTY harness intercepts cursor position queries automatically
   - If seeing escape sequences in output, may need additional interception
   - Check `intercept_control_sequences()` in lib.rs

## Test File Organization

Place tests in `codex-rs/tui-integration-tests/tests/`, for example:

| File | Coverage |
|------|----------|
| `startup.rs` | TUI initialization, welcome screens, trust screen handling |
| `prompt_flow.rs` | Prompt submission, agent responses, multiline input |
| `input_handling.rs` | Text editing, backspace, keyboard events |
| `cancellation.rs` | Stream cancellation, Ctrl-C, Escape handling |

Create new test files for distinct feature areas (e.g., `markdown_rendering.rs`, `command_approval.rs`, etc.)

## Example: Full Test Implementation

```rust
use insta::assert_snapshot;
use std::time::Duration;
use tui_integration_tests::{Key, SessionConfig, TuiSession};

const TIMEOUT: Duration = Duration::from_secs(10);

#[test]
fn test_complete_interaction_flow() {
    // Configure mock agent with custom response
    let config = SessionConfig::new()
        .with_mock_response("I can help with that task.");

    // Spawn in larger terminal for better layout testing
    let mut session = TuiSession::spawn_with_config(40, 120, config)
        .expect("Failed to spawn codex");

    // Wait for initial prompt
    session.wait_for_text("To get started", TIMEOUT)
        .expect("Initial prompt did not appear");

    // Simulate user interaction
    session.send_str("Please help me").unwrap();
    session.wait_for_text("Please help me", TIMEOUT).unwrap();

    // Submit prompt
    session.send_key(Key::Enter).unwrap();

    // Wait for agent response
    session.wait_for_text("I can help with that task", TIMEOUT)
        .expect("Agent response did not appear");

    // Capture final state for regression testing
    assert_snapshot!("complete_interaction", session.screen_contents());
}
```

## Testing Philosophy

**Black-box Integration:**

- Tests exercise the full application stack (CLI → TUI → Core → ACP)
- No direct access to TUI internals, validates external behavior only
- Screen content assertions mirror real user experience

**Isolation:**

- Each test runs in isolated temporary directory
- No shared state between tests
- Automatic cleanup on test completion

**Determinism:**

- Mock agent provides predictable responses
- NO_COLOR=1 disables color codes
- Fixed terminal dimensions
- Snapshot testing catches unintended regressions

**Complementary to Unit Tests:**

- Unit tests in `codex-rs/tui/src/` validate component logic
- Integration tests validate end-to-end terminal rendering and interaction
- Both are necessary for comprehensive coverage
