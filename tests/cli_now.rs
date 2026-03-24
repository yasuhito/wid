#![allow(dead_code)]

#[allow(dead_code)]
#[path = "../src/log/model.rs"]
mod model;
#[allow(dead_code)]
#[path = "../src/log/format.rs"]
mod format;
#[path = "../src/log/parser.rs"]
mod parser;
#[path = "../src/log/paths.rs"]
mod paths;
#[path = "../src/log/store.rs"]
mod store;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
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

    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();

    if let Some(input) = stdin {
        child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    }

    child.wait_with_output().unwrap()
}

fn log_path(home: &PathBuf) -> PathBuf {
    home.join(".local/share/wid/log.md")
}

fn log_contents(home: &PathBuf) -> String {
    fs::read_to_string(log_path(home)).unwrap()
}

#[test]
fn now_command_joins_remaining_args_with_spaces() {
    let home = unique_temp_dir("now-join");
    let output = run_wid(&home, &["now", "CI", "が", "落ちていたので修正"], None);

    assert!(output.status.success(), "{output:?}");
    let contents = log_contents(&home);
    assert!(contents.contains("CI が 落ちていたので修正"), "{contents}");
}

#[test]
fn now_command_reads_single_line_when_no_args_are_given() {
    let home = unique_temp_dir("now-stdin");
    let output = run_wid(&home, &["now"], Some("レビュー指摘を反映\n"));

    assert!(output.status.success(), "{output:?}");
    let contents = log_contents(&home);
    assert!(contents.contains("レビュー指摘を反映"), "{contents}");
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
fn append_log_entry_creates_new_file_with_header_today_section_and_entry() {
    let dir = unique_temp_dir("append-create");
    let path = dir.join(".local/share/wid/log.md");

    store::append_log_entry(&path, "2026-03-24", "11:32", "CI が落ちていたので修正")
        .unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n"
    );
}

#[test]
fn append_log_entry_reuses_existing_same_day_section() {
    let dir = unique_temp_dir("append-reuse");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "実装方針を見直した").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n"
    );
}

#[test]
fn append_log_entry_reuses_matching_section_even_when_it_is_not_last() {
    let dir = unique_temp_dir("append-reuse-middle");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n\n## 2026-03-25\n\n- 09:00 別件を開始\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "実装方針を見直した").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n\n## 2026-03-25\n\n- 09:00 別件を開始\n"
    );
}

#[test]
fn append_log_entry_preserves_unrelated_text_around_matching_section() {
    let dir = unique_temp_dir("append-preserve-text");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\nintro note\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n\nmisc note\n\n## 2026-03-25\n\n- 09:00 別件を開始\n\ntrailing note\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "実装方針を見直した").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\nintro note\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n\nmisc note\n- 12:10 実装方針を見直した\n\n## 2026-03-25\n\n- 09:00 別件を開始\n\ntrailing note\n"
    );
}

#[test]
fn append_log_entry_reuses_heading_with_crlf_and_trailing_spaces() {
    let dir = unique_temp_dir("append-crlf-heading");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\r\n\r\n## 2026-03-24   \r\n\r\n- 11:32 CI が落ちていたので修正\r\n\r\n## 2026-03-25\r\n\r\n- 09:00 別件を開始\r\n",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "実装方針を見直した").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\r\n\r\n## 2026-03-24   \r\n\r\n- 11:32 CI が落ちていたので修正\r\n- 12:10 実装方針を見直した\r\n\r\n## 2026-03-25\r\n\r\n- 09:00 別件を開始\r\n"
    );
}

#[test]
fn append_log_entry_inserts_newline_before_appending_at_eof_without_trailing_newline() {
    let dir = unique_temp_dir("append-eof-no-trailing-newline");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正",
    )
    .unwrap();

    store::append_log_entry(&path, "2026-03-24", "12:10", "実装方針を見直した").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n"
    );
}
