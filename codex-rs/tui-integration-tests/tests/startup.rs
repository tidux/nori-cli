use insta::assert_snapshot;
use std::time::Duration;
use tui_integration_tests::TuiSession;

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn test_startup_shows_prompt() {
    let mut session = TuiSession::spawn(24, 80).expect("Failed to spawn codex");

    session
        .wait_for_text("Welcome", TIMEOUT)
        .expect("Prompt did not appear");

    assert_snapshot!("startup_screen", session.screen_contents());
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
