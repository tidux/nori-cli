use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use vt100::Parser;

pub use keys::Key;
mod keys;

/// PTY session for driving the codex TUI
pub struct TuiSession {
    master: Box<dyn portable_pty::MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    parser: Parser,
}

impl TuiSession {
    /// Spawn codex with mock-acp-agent
    pub fn spawn(rows: u16, cols: u16) -> Result<Self> {
        Self::spawn_with_config(rows, cols, SessionConfig::default())
    }

    /// Spawn with custom configuration
    pub fn spawn_with_config(rows: u16, cols: u16, config: SessionConfig) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(codex_binary_path());

        // Use mock-acp-agent model
        cmd.arg("--model");
        cmd.arg(&config.model);

        // Set TERM to enable terminal features
        cmd.env("TERM", "xterm-256color");

        // Pass through mock agent env vars
        for (key, value) in config.mock_agent_env {
            cmd.env(&key, &value);
        }

        // Disable color codes for easier parsing
        if config.no_color {
            cmd.env("NO_COLOR", "1");
        }

        let _child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok(Self {
            master: pair.master,
            reader,
            writer,
            parser: Parser::new(rows, cols, 0),
        })
    }

    /// Read any available output and update screen state
    ///
    /// This method attempts to read available data without blocking.
    /// It uses a simple approach of reading with a small buffer which works
    /// well for our polling-based test framework.
    pub fn poll(&mut self) -> Result<()> {
        // Create a small buffer for reading
        let mut buf = [0u8; 8192];

        // The PTY reader will return immediately if no data is available
        // We rely on the polling loop in wait_for() to handle timing
        match self.reader.read(&mut buf) {
            Ok(0) => {
                // EOF - process exited
                Ok(())
            }
            Ok(n) => {
                // Intercept and respond to control sequences before parsing
                let processed = self.intercept_control_sequences(&buf[..n])?;
                self.parser.process(&processed);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available right now
                Ok(())
            }
            Err(e) => {
                // Actual error
                Err(e.into())
            }
        }
    }

    /// Intercept control sequences and inject responses
    ///
    /// Detects cursor position queries (ESC[6n) and writes responses back to the PTY
    /// Returns filtered data with control sequences removed
    fn intercept_control_sequences(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(data.len());
        let mut i = 0;

        while i < data.len() {
            // Detect cursor position query: ESC[6n
            if i + 3 < data.len()
                && data[i] == 0x1b      // ESC
                && data[i+1] == b'['
                && data[i+2] == b'6'
                && data[i+3] == b'n'
            {
                // Write response back to PTY: ESC[1;1R (cursor at row 1, col 1)
                self.writer.write_all(b"\x1b[1;1R")?;
                self.writer.flush()?;
                // Skip the control sequence - don't pass it to the parser
                i += 4;
            } else {
                result.push(data[i]);
                i += 1;
            }
        }
        Ok(result)
    }

    /// Wait for predicate with timeout
    pub fn wait_for<F>(&mut self, pred: F, timeout: Duration) -> Result<(), String>
    where
        F: Fn(&str) -> bool,
    {
        let start = Instant::now();
        loop {
            self.poll().map_err(|e| e.to_string())?;
            let contents = self.screen_contents();
            if pred(&contents) {
                return Ok(());
            }
            if start.elapsed() > timeout {
                return Err(format!(
                    "Timeout waiting for condition.\nScreen contents:\n{}",
                    contents
                ));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Wait for specific text to appear
    pub fn wait_for_text(&mut self, needle: &str, timeout: Duration) -> Result<(), String> {
        self.wait_for(|s| s.contains(needle), timeout)
    }

    /// Get current screen contents as string
    pub fn screen_contents(&self) -> String {
        self.parser.screen().contents()
    }

    /// Type a string
    pub fn send_str(&mut self, s: &str) -> std::io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()
    }

    /// Send a key event
    pub fn send_key(&mut self, key: Key) -> std::io::Result<()> {
        self.writer.write_all(&key.to_escape_sequence())?;
        self.writer.flush()
    }
}

/// Configuration for spawning a test session
#[derive(Default)]
pub struct SessionConfig {
    pub model: String,
    pub mock_agent_env: HashMap<String, String>,
    pub no_color: bool,
}

impl SessionConfig {
    pub fn new() -> Self {
        Self {
            model: "mock-acp-agent".to_string(),
            mock_agent_env: HashMap::new(),
            no_color: true,
        }
    }

    pub fn with_mock_response(mut self, response: impl Into<String>) -> Self {
        self.mock_agent_env
            .insert("MOCK_AGENT_RESPONSE".to_string(), response.into());
        self
    }

    pub fn with_stream_until_cancel(mut self) -> Self {
        self.mock_agent_env.insert(
            "MOCK_AGENT_STREAM_UNTIL_CANCEL".to_string(),
            "1".to_string(),
        );
        self
    }

    pub fn with_agent_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.mock_agent_env.insert(key.into(), value.into());
        self
    }
}

/// Get path to codex binary
fn codex_binary_path() -> String {
    let test_exe = std::env::current_exe().expect("Failed to get current exe path");
    test_exe
        .parent() // deps
        .and_then(|p| p.parent()) // debug or release
        .expect("Failed to get target directory")
        .join("codex")
        .to_string_lossy()
        .into_owned()
}
