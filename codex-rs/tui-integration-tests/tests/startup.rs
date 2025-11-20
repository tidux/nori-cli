use std::time::Duration;
use tui_integration_tests::TuiSession;

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn test_startup_shows_prompt() {
    let mut session = TuiSession::spawn(24, 80).expect("Failed to spawn codex");

    session
        .wait_for_text("Welcome", TIMEOUT)
        .expect("Prompt did not appear");

    let contents = session.screen_contents();
    assert!(contents.contains("Welcome to Codex"));
    assert!(contents.contains("/tmp/"));
}

#[test]
fn test_startup_with_dimensions() {
    let mut session = TuiSession::spawn(40, 120).expect("Failed to spawn codex");

    session
        .wait_for_text("Welcome", TIMEOUT)
        .expect("Prompt did not appear");

    // Verify terminal size is respected
    let contents = session.screen_contents();
    assert!(contents.lines().count() <= 40);
}

#[test]
fn test_runs_in_temp_directory_by_default() {
    let mut session = TuiSession::spawn(24, 80).expect("Failed to spawn codex");

    session
        .wait_for_text("Welcome", TIMEOUT)
        .expect("Prompt did not appear");

    let contents = session.screen_contents();

    // Should run in /tmp/, not home directory
    assert!(
        contents.contains("/tmp/"),
        "Expected session to run in /tmp/, but got: {}",
        contents
    );

    // Should NOT run in home directory
    assert!(
        !contents.contains("/home/"),
        "Session should not run in home directory, but got: {}",
        contents
    );
}
