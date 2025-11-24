# Noridoc: ACP Module

Path: @/codex-rs/acp

## Overview

- Implements Agent Context Protocol (ACP) for Codex to communicate with external AI agent subprocesses
- Uses the official `agent-client-protocol` v0.7 library instead of any custom JSON-RPC implementation
- Exports `init_file_tracing()` for file-based structured logging at DEBUG level

### How it fits into the larger codebase

- Used by `@/codex-rs/core/src/client.rs` to communicate with ACP-compliant agents via `WireApi::Acp` variant
- Uses channel-based streaming pattern (mpsc) consistent with core's `ResponseStream`
- Provides structured error handling via library's typed error responses that core translates to user-facing messages
- TUI and other clients can access captured stderr for displaying agent diagnostic output

### Model Registry

The ACP registry in `@/codex-rs/acp/src/registry.rs` is **model-centric** rather than provider-centric:
- `get_agent_config()` accepts model names (e.g., "mock-model", "gemini-2.5-flash") instead of provider names
- Called from `@/codex-rs/core/src/client.rs` at the start of `stream()` to check if model is an ACP agent
- Returns `AcpAgentConfig` containing:
  - `provider_slug`: Identifies which agent subprocess to spawn (e.g., "mock-acp", "gemini-acp")
  - `command`: Executable path or command name
  - `args`: Arguments to pass to the subprocess
  - `provider_info`: Embedded `AcpProviderInfo` with provider configuration (name, retry settings, timeouts)
- Model names are normalized to lowercase for case-insensitive matching (e.g., "Gemini-2.5-Flash" → "gemini-2.5-flash")
- Uses exact matching only (no prefix matching) - each model must be explicitly registered
- The `provider_slug` field enables future optimization to determine when existing subprocess can be reused vs when new one must be spawned when switching models

### Embedded Provider Info

ACP providers embed their configuration directly in `AcpAgentConfig` via `AcpProviderInfo`:
- Avoids circular dependency between `codex-acp` and `codex-core` (core depends on acp, not vice versa)
- ACP providers are NOT in `built_in_model_providers()` in core - they're self-contained in the registry
- `AcpProviderInfo` contains:
  - `name`: Display name (e.g., "Gemini ACP")
  - `request_max_retries`: Max request retries (default: 1)
  - `stream_max_retries`: Max stream reconnection attempts (default: 1)
  - `stream_idle_timeout`: Idle timeout for streaming (default: 5 minutes)
- Core's `client.rs` checks the ACP registry first in `stream()`, using the embedded provider info for ACP models


### Stderr Capture Implementation

- Buffer lines per session for access between reader task and caller
- Bounded at 500 lines with FIFO eviction when full
- Individual lines truncated to 10KB
- Reader task runs until EOF or error, logging warnings via tracing

### File-Based Tracing

The `init_file_tracing()` function in `@/codex-rs/acp/src/tracing_setup.rs` provides structured file logging:
- Sets global tracing subscriber that writes to a user-specified file path
- Filters at DEBUG level and above (TRACE is excluded)
- Uses non-blocking file appender for async-safe writes
- Creates parent directories automatically if they don't exist
- Returns error on re-initialization since global subscriber can only be set once per process
- Guard is intentionally leaked via `std::mem::forget()` to keep non-blocking writer alive for program lifetime
- ANSI colors disabled for clean file output
- Automatically initialized by the CLI (`@/codex-rs/cli/src/main.rs`) at startup, writing to `.codex-acp.log` in the current working directory

### Core Implementation

TODO!

Created and maintained by Nori.
