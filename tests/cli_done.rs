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
#[path = "../src/interactive/done_picker.rs"]
mod done_picker;
mod interactive {
    pub mod done_picker {
        pub use crate::done_picker::*;
    }
}
mod log {
    pub mod model {
        pub use crate::model::*;
    }
    pub mod paths {
        pub use crate::paths::*;
    }
    pub mod store {
        pub use crate::store::*;
    }
}
#[path = "../src/commands/done.rs"]
mod done_command;

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
fn done_interactive_picker_moves_down_and_confirms_selection() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut picker = interactive::done_picker::PickerState::new(3);
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Continue
    );
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Confirmed(1)
    );
}

#[test]
fn done_interactive_picker_supports_k_and_up_for_previous_selection() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut picker = interactive::done_picker::PickerState::new(3);
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Continue
    );
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Continue
    );
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Continue
    );
    assert_eq!(picker.selected(), 1);
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Continue
    );
    assert_eq!(picker.selected(), 0);
}

#[test]
fn done_interactive_picker_cancels_on_q_and_escape() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut picker = interactive::done_picker::PickerState::new(2);
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Cancelled
    );

    let mut picker = interactive::done_picker::PickerState::new(2);
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        interactive::done_picker::PickerOutcome::Cancelled
    );
}

#[test]
fn done_command_interactive_marks_selected_entry() {
    let dir = unique_temp_dir("done-interactive-mark-selected");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n- 11:48 already done @done(2026-03-24 11:50)\n\n## 2026-03-25\n\n- 09:15 selected item\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(0));
    done_command::run_interactive_at_path(&path, "2026-03-25 09:16", &mut picker).unwrap();

    assert_eq!(picker.calls, 1);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n- 11:48 already done @done(2026-03-24 11:50)\n\n## 2026-03-25\n\n- 09:15 selected item @done(2026-03-25 09:16)\n"
    );
}

#[test]
fn done_command_interactive_cancel_leaves_log_unchanged() {
    let dir = unique_temp_dir("done-interactive-cancel");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n- 11:48 second unfinished\n",
    )
    .unwrap();

    let before = fs::read_to_string(&path).unwrap();
    let mut picker = FakePicker::new(None);
    done_command::run_interactive_at_path(&path, "2026-03-24 11:52", &mut picker).unwrap();

    assert_eq!(picker.calls, 1);
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn done_command_interactive_errors_on_out_of_bounds_selection() {
    let dir = unique_temp_dir("done-interactive-out-of-bounds");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 only unfinished\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(9));
    let error =
        done_command::run_interactive_at_path(&path, "2026-03-24 11:52", &mut picker).unwrap_err();

    assert!(format!("{error:#}").contains("selection"), "{error:#}");
}

#[test]
fn done_store_rejects_stale_unfinished_entry_targets() {
    let dir = unique_temp_dir("done-stale-target");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 only unfinished\n",
    )
    .unwrap();

    let target = store::collect_unfinished_entries(&path).unwrap().pop().unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 only unfinished (renamed)\n",
    )
    .unwrap();

    let error = store::mark_unfinished_entry_done(&path, &target, "2026-03-24 11:52").unwrap_err();

    assert!(format!("{error:#}").contains("changed"), "{error:#}");
}

#[test]
fn done_store_handles_trailing_spaces_like_the_parser() {
    let dir = unique_temp_dir("done-trailing-spaces");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24   \n\n- 11:32 spaced entry   \n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24   \n\n- 11:32 spaced entry    @done(2026-03-24 11:52)\n"
    );
}

#[test]
fn done_store_collects_unfinished_entries_in_oldest_first_order() {
    let dir = unique_temp_dir("done-collect-order");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n\n## 2026-03-25\n\n- 09:15 latest unfinished\n",
    )
    .unwrap();

    let entries = store::collect_unfinished_entries(&path).unwrap();
    let labels: Vec<_> = entries.iter().map(|entry| entry.display_label()).collect();

    assert_eq!(
        labels,
        vec![
            "2026-03-24 11:32 first unfinished".to_string(),
            "2026-03-25 09:15 latest unfinished".to_string(),
        ]
    );
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

struct FakePicker {
    result: Option<usize>,
    calls: usize,
}

impl FakePicker {
    fn new(result: Option<usize>) -> Self {
        Self { result, calls: 0 }
    }
}

impl interactive::done_picker::Picker for FakePicker {
    fn pick<T: model::PickerItem>(&mut self, _entries: &[T]) -> anyhow::Result<Option<usize>> {
        self.calls += 1;
        Ok(self.result)
    }
}
