#![allow(dead_code, unused_imports)]

#[allow(dead_code)]
#[path = "../src/log/format.rs"]
mod format;
#[allow(dead_code)]
#[path = "../src/log/model.rs"]
mod model;
#[path = "../src/log/parser.rs"]
mod parser;
#[path = "../src/log/paths.rs"]
mod paths;
#[path = "../src/commands/show.rs"]
mod show_command;
#[path = "../src/log/store.rs"]
mod store;
mod log {
    pub mod model {
        pub use crate::model::*;
    }
    pub mod store {
        pub use crate::store::*;
    }
}
mod commands {
    pub mod show {
        pub use crate::show_command::*;
    }
}

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    dir.push(format!("wid-{name}-{stamp}-{}", std::process::id()));
    dir
}

fn wid_bin() -> &'static str {
    env!("CARGO_BIN_EXE_wid")
}

fn run_wid(home: &PathBuf, args: &[&str], stdin: Option<&str>) -> std::process::Output {
    let mut command = Command::new(wid_bin());
    command.env("HOME", home).args(args);

    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(input) = stdin {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }

    child.wait_with_output().unwrap()
}

fn log_path(home: &Path) -> PathBuf {
    home.join(".local/share/wid/log.md")
}

fn log_contents(home: &Path) -> String {
    fs::read_to_string(log_path(home)).unwrap()
}

#[test]
fn now_command_joins_remaining_args_with_spaces() {
    let home = unique_temp_dir("now-join");
    let output = run_wid(&home, &["now", "fix", "failing", "CI"], None);

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("## "), "{stdout}");
    assert!(stdout.contains("- [>] "), "{stdout}");
    assert!(stdout.contains("fix failing CI"), "{stdout}");
    let contents = log_contents(&home);
    assert!(contents.contains("fix failing CI"), "{contents}");
}

#[test]
fn now_command_reads_single_line_when_no_args_are_given() {
    let home = unique_temp_dir("now-stdin");
    let output = run_wid(&home, &["now"], Some("address review feedback\n"));

    assert!(output.status.success(), "{output:?}");
    let contents = log_contents(&home);
    assert!(contents.contains("address review feedback"), "{contents}");
}

#[test]
fn now_command_rejects_empty_input_line() {
    let home = unique_temp_dir("now-empty-stdin");
    let output = run_wid(&home, &["now"], Some("\n"));

    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty"), "{stderr}");
}

#[test]
fn now_command_rejects_empty_and_whitespace_args() {
    for (name, args) in [
        ("now-empty-arg", vec!["now", ""]),
        ("now-whitespace-arg", vec!["now", "   "]),
    ] {
        let home = unique_temp_dir(name);
        let output = run_wid(&home, &args, None);

        assert!(!output.status.success(), "{output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("empty"), "{stderr}");
    }
}

#[test]
fn append_log_entry_creates_new_file_with_today_section_and_entry() {
    let dir = unique_temp_dir("append-create");
    let path = dir.join(".local/share/wid/log.md");

    store::append_log_entry(&path, "2026-03-24", "11:32", "fix failing CI").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24\n\n- [>] 11:32 fix failing CI\n"
    );
}

#[test]
fn append_log_entry_reuses_existing_same_day_section() {
    let dir = unique_temp_dir("append-reuse");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-24\n\n- [>] 11:32 fix failing CI\n").unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "rework implementation plan").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [>] 12:10 rework implementation plan\n"
    );
}

#[test]
fn append_log_entry_reuses_matching_section_even_when_it_is_not_last() {
    let dir = unique_temp_dir("append-reuse-middle");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-24\n\n- [>] 11:32 fix failing CI\n\n## 2026-03-25\n\n- [ ] 09:00 start another task\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "rework implementation plan").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [>] 12:10 rework implementation plan\n\n## 2026-03-25\n\n- [ ] 09:00 start another task\n"
    );
}

#[test]
fn append_log_entry_preserves_unrelated_text_around_matching_section() {
    let dir = unique_temp_dir("append-preserve-text");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "intro note\n\n## 2026-03-24\n\n- [>] 11:32 fix failing CI\n\nmisc note\n\n## 2026-03-25\n\n- [ ] 09:00 start another task\n\ntrailing note\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "rework implementation plan").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "intro note\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n\nmisc note\n- [>] 12:10 rework implementation plan\n\n## 2026-03-25\n\n- [ ] 09:00 start another task\n\ntrailing note\n"
    );
}

#[test]
fn append_log_entry_reuses_heading_with_crlf_and_trailing_spaces() {
    let dir = unique_temp_dir("append-crlf-heading");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-24   \r\n\r\n- [>] 11:32 fix failing CI\r\n\r\n## 2026-03-25\r\n\r\n- [ ] 09:00 start another task\r\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "rework implementation plan").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24   \r\n\r\n- [ ] 11:32 fix failing CI\r\n- [>] 12:10 rework implementation plan\r\n\r\n## 2026-03-25\r\n\r\n- [ ] 09:00 start another task\r\n"
    );
}

#[test]
fn append_log_entry_inserts_newline_before_appending_at_eof_without_trailing_newline() {
    let dir = unique_temp_dir("append-eof-no-trailing-newline");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-24\n\n- [>] 11:32 fix failing CI").unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "rework implementation plan").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [>] 12:10 rework implementation plan\n"
    );
}
