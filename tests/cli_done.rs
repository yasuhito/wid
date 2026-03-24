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

fn write_log(home: &PathBuf, contents: &str) {
    let path = log_path(home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn done_command_marks_last_unfinished_entry() {
    let home = unique_temp_dir("done-mark-last");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)\n- 12:10 実装方針を見直した\n",
    );

    let output = run_wid(&home, &["done"], None);

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(contents.contains("- 12:10 実装方針を見直した @done("), "{contents}");
    assert!(contents.contains("- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)"), "{contents}");
}

#[test]
fn done_command_skips_already_done_trailing_entries() {
    let home = unique_temp_dir("done-skip-trailing");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)\n- 12:10 実装方針を見直した @done(2026-03-24 11:51)\n",
    );

    let output = run_wid(&home, &["done"], None);

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(contents.contains("- 11:32 CI が落ちていたので修正 @done("), "{contents}");
    assert!(contents.contains("- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)"), "{contents}");
    assert!(contents.contains("- 12:10 実装方針を見直した @done(2026-03-24 11:51)"), "{contents}");
}

#[test]
fn done_command_errors_when_no_unfinished_entry_exists() {
    let home = unique_temp_dir("done-no-open-entry");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正 @done(2026-03-24 11:50)\n",
    );

    let output = run_wid(&home, &["done"], None);

    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unfinished"), "{stderr}");
}

#[test]
fn done_command_errors_when_log_has_no_entries() {
    let home = unique_temp_dir("done-empty-log");
    let output = run_wid(&home, &["done"], None);

    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unfinished"), "{stderr}");
}

#[test]
fn done_store_updates_last_unfinished_entry_in_place() {
    let dir = unique_temp_dir("done-store-mark-last");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)\n- 12:10 実装方針を見直した\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:50").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)\n- 12:10 実装方針を見直した @done(2026-03-24 11:50)\n"
    );
}

#[test]
fn done_store_skips_trailing_done_entries() {
    let dir = unique_temp_dir("done-store-skip-trailing");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)\n- 12:10 実装方針を見直した @done(2026-03-24 11:51)\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正 @done(2026-03-24 11:52)\n- 11:48 レビュー指摘を反映 @done(2026-03-24 11:50)\n- 12:10 実装方針を見直した @done(2026-03-24 11:51)\n"
    );
}

#[test]
fn done_store_treats_interior_done_marker_as_unfinished() {
    let dir = unique_temp_dir("done-store-interior-marker");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 @done( mentions a topic, but this is still unfinished\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 @done( mentions a topic, but this is still unfinished @done(2026-03-24 11:52)\n"
    );
}

#[test]
fn done_store_ignores_later_non_entry_bullet_lines() {
    let dir = unique_temp_dir("done-store-ignore-note");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n- note: follow-up detail from the same day\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した @done(2026-03-24 11:52)\n- note: follow-up detail from the same day\n"
    );
}

#[test]
fn done_store_ignores_entry_looking_lines_before_day_section() {
    let dir = unique_temp_dir("done-store-before-section");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 12:10 実装方針を見直した\n\n## Notes\n\n- 11:32 outside any day section\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 12:10 実装方針を見直した @done(2026-03-24 11:52)\n\n## Notes\n\n- 11:32 outside any day section\n"
    );
}

#[test]
fn done_store_errors_when_no_unfinished_entry_exists() {
    let dir = unique_temp_dir("done-store-empty");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正 @done(2026-03-24 11:50)\n",
    )
    .unwrap();

    let error = store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap_err();
    let message = format!("{error:#}");

    assert!(message.contains("unfinished"), "{message}");
}
