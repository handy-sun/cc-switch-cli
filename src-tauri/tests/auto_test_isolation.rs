use std::path::PathBuf;

use cc_switch_lib::{get_app_config_dir, get_claude_mcp_path};

#[test]
fn cargo_test_uses_isolated_home_without_per_test_setup() {
    let expected_home =
        std::env::temp_dir().join(format!("cc-switch-test-home-{}", std::process::id()));

    assert_eq!(
        std::env::var_os("HOME").map(PathBuf::from),
        Some(expected_home.clone())
    );
    assert_eq!(std::env::var_os("CC_SWITCH_TUI_CONFIG_DIR"), None);
    assert_eq!(std::env::var_os("CC_SWITCH_CONFIG_DIR"), None);
    assert_eq!(get_app_config_dir(), expected_home.join(".cc-switch-tui"));
    assert_eq!(get_claude_mcp_path(), expected_home.join(".claude.json"));
}
