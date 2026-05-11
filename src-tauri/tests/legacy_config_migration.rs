use serial_test::serial;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
#[serial]
fn approved_legacy_migration_runs_before_custom_config_db_creation() {
    let home = TempDir::new().expect("create temp home");
    let old_dir = home.path().join(".cc-switch");
    let new_dir = home.path().join(".config").join("cc-switch-tui");

    fs::create_dir_all(old_dir.join("skills")).expect("create legacy config");
    let legacy_config =
        serde_json::to_string_pretty(&cc_switch_lib::MultiAppConfig::default()).unwrap();
    fs::write(old_dir.join("config.json"), legacy_config).expect("write legacy config");
    fs::write(old_dir.join("skills").join("demo.md"), "# Demo").expect("write legacy skill");

    let mut child = Command::new(env!("CARGO_BIN_EXE_cc-switch-tui"))
        .args(["config", "path"])
        .env("HOME", home.path())
        .env("CC_SWITCH_TUI_CONFIG_DIR", &new_dir)
        .env_remove("CC_SWITCH_CONFIG_DIR")
        .env_remove("CLAUDE_CONFIG_DIR")
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run cc-switch");

    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"y\n")
        .expect("approve migration");

    let output = child.wait_with_output().expect("wait for cc-switch");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        new_dir.join("cc-switch.db").exists(),
        "database should exist"
    );
    assert!(
        new_dir.join("config.json.migrated").exists(),
        "legacy config should be copied before DB migration archives it"
    );
    assert!(
        new_dir.join("skills").join("demo.md").exists(),
        "legacy directories should be copied, not just the database"
    );
    assert!(
        new_dir.join(".migrated-from-cc-switch").exists(),
        "migration marker should be written"
    );
}
