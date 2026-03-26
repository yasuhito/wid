#![allow(dead_code, unused_imports)]

#[path = "../src/interactive/done_picker.rs"]
mod done_picker;
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
mod interactive {
    pub mod done_picker {
        pub use crate::done_picker::*;
    }
}
mod commands {
    pub mod show {
        pub use crate::show_command::*;
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

fn write_log(home: &Path, contents: &str) {
    let path = log_path(home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn done_command_marks_last_unfinished_entry() {
    let home = unique_temp_dir("done-mark-last");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 11:48 address review feedback\n- [ ] 12:10 rework implementation plan\n",
    );

    let output = run_wid(&home, &["done"], None);

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(
        contents.contains("- [x] 12:10 rework implementation plan"),
        "{contents}"
    );
    assert!(
        contents.contains("- [x] 11:48 address review feedback"),
        "{contents}"
    );
}

#[test]
fn done_command_marks_entry_by_transient_id() {
    let home = unique_temp_dir("done-by-id");
    write_log(
        &home,
        "## 2026-03-24\n\n- [ ] 11:32 first item\n- [ ] 11:48 target item\n",
    );

    let json_output = run_wid(&home, &["--json"], None);
    assert!(json_output.status.success(), "{json_output:?}");
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][1]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["done", "--id", &id], None);

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(contents.contains("- [x] 11:48 target item"), "{contents}");
}

#[test]
fn done_command_by_id_is_noop_when_target_is_already_done() {
    let home = unique_temp_dir("done-by-id-noop");
    write_log(&home, "## 2026-03-24\n\n- [x] 11:32 already done\n");

    let json_output = run_wid(&home, &["--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let before = fs::read_to_string(log_path(&home)).unwrap();
    let output = run_wid(&home, &["done", "--id", &id], None);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(fs::read_to_string(log_path(&home)).unwrap(), before);
}

#[test]
fn done_command_by_id_errors_when_item_changed() {
    let home = unique_temp_dir("done-by-id-stale");
    write_log(&home, "## 2026-03-24\n\n- [ ] 11:32 target item\n");

    let json_output = run_wid(&home, &["--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    write_log(&home, "## 2026-03-24\n\n- [ ] 11:32 target item (edited)\n");

    let output = run_wid(&home, &["done", "--id", &id], None);

    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("changed or not found"), "{stderr}");
}

#[test]
fn done_command_marks_active_entry_first() {
    let home = unique_temp_dir("done-mark-active-first");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [>] 12:10 rework implementation plan\n",
    );

    let output = run_wid(&home, &["done"], None);

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("☑ rework implementation plan  12:10"),
        "{stdout}"
    );
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(
        contents.contains("- [ ] 11:32 fix failing CI"),
        "{contents}"
    );
    assert!(
        contents.contains("- [x] 12:10 rework implementation plan"),
        "{contents}"
    );
}

#[test]
fn done_command_skips_already_done_trailing_entries() {
    let home = unique_temp_dir("done-skip-trailing");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 11:48 address review feedback\n- [x] 12:10 rework implementation plan\n",
    );

    let output = run_wid(&home, &["done"], None);

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(
        contents.contains("- [x] 11:32 fix failing CI"),
        "{contents}"
    );
    assert!(
        contents.contains("- [x] 11:48 address review feedback"),
        "{contents}"
    );
    assert!(
        contents.contains("- [x] 12:10 rework implementation plan"),
        "{contents}"
    );
}

#[test]
fn done_command_errors_when_no_unfinished_entry_exists() {
    let home = unique_temp_dir("done-no-open-entry");
    write_log(
        &home,
        "# wid log\n\n## 2026-03-24\n\n- [x] 11:32 fix failing CI\n",
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
fn done_interactive_picker_toggles_selected_entry_on_space() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut picker = interactive::done_picker::DonePickerState::new(vec![
        model::EntryState::Pending,
        model::EntryState::Active,
        model::EntryState::Done,
    ]);

    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        interactive::done_picker::DonePickerOutcome::Continue
    );
    assert_eq!(picker.states()[0], model::EntryState::Done);

    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
        interactive::done_picker::DonePickerOutcome::Continue
    );
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        interactive::done_picker::DonePickerOutcome::Continue
    );
    assert_eq!(picker.states()[1], model::EntryState::Done);

    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
        interactive::done_picker::DonePickerOutcome::Continue
    );
    assert_eq!(
        picker.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        interactive::done_picker::DonePickerOutcome::Continue
    );
    assert_eq!(picker.states()[2], model::EntryState::Pending);
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
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first unfinished\n- [x] 11:48 already done\n\n## 2026-03-25\n\n- [ ] 09:15 selected item\n",
    )
    .unwrap();

    let mut picker = FakeDonePicker::new(Some(vec![
        model::EntryState::Pending,
        model::EntryState::Done,
        model::EntryState::Done,
    ]));
    done_command::run_interactive_at_path(&path, "2026-03-25 09:16", &mut picker).unwrap();

    assert_eq!(picker.calls, 1);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first unfinished\n- [x] 11:48 already done\n\n## 2026-03-25\n\n- [x] 09:15 selected item\n"
    );
}

#[test]
fn done_command_interactive_lists_unfinished_entries_in_wid_order() {
    let dir = unique_temp_dir("done-interactive-order");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 oldest item\n- [x] 11:48 already done\n\n## 2026-03-25\n\n- [ ] 09:15 newest unfinished\n",
    )
    .unwrap();

    let mut picker = FakeDonePicker::new(None);
    done_command::run_interactive_at_path(&path, "2026-03-25 09:16", &mut picker).unwrap();

    assert_eq!(picker.calls, 1);
    assert_eq!(
        picker.items,
        vec![
            "2026-03-24 Tue".to_string(),
            "─".repeat("2026-03-24 Tue".chars().count()),
            "□ oldest item  11:32".to_string(),
            "☑ already done  11:48".to_string(),
            " ".to_string(),
            "Yesterday · 2026-03-25 Wed".to_string(),
            "─".repeat("Yesterday · 2026-03-25 Wed".chars().count()),
            "□ newest unfinished  09:15".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn done_command_interactive_lists_active_entry_and_selects_it_by_default() {
    let dir = unique_temp_dir("done-interactive-active");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 pending task\n\n## 2026-03-25\n\n- [x] 09:15 done item\n",
    )
    .unwrap();

    let mut picker = FakeDonePicker::new(None);
    done_command::run_interactive_at_path(&path, "2026-03-25 09:16", &mut picker).unwrap();

    assert_eq!(
        picker.items,
        vec![
            "2026-03-24 Tue".to_string(),
            "─".repeat("2026-03-24 Tue".chars().count()),
            "◉ current task  11:32".to_string(),
            "□ pending task  11:48".to_string(),
            " ".to_string(),
            "Yesterday · 2026-03-25 Wed".to_string(),
            "─".repeat("Yesterday · 2026-03-25 Wed".chars().count()),
            "☑ done item  09:15".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn done_command_interactive_shows_notes_under_entries() {
    let dir = unique_temp_dir("done-interactive-notes");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-24\n\n- [>] 11:32 current task\n  - first note\n  - second note\n\n- [ ] 11:48 pending task\n  - follow-up detail\n",
    )
    .unwrap();

    let mut picker = FakeDonePicker::new(None);
    done_command::run_interactive_at_path(&path, "2026-03-25 09:16", &mut picker).unwrap();

    assert_eq!(
        picker.items,
        vec![
            "2026-03-24 Tue".to_string(),
            "─".repeat("2026-03-24 Tue".chars().count()),
            "◉ current task  11:32".to_string(),
            "  · first note".to_string(),
            "  · second note".to_string(),
            "□ pending task  11:48".to_string(),
            "  · follow-up detail".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn done_command_interactive_marks_selected_active_entry_done() {
    let dir = unique_temp_dir("done-interactive-active-selected");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 pending task\n",
    )
    .unwrap();

    let mut picker = FakeDonePicker::new(Some(vec![
        model::EntryState::Done,
        model::EntryState::Pending,
    ]));
    done_command::run_interactive_at_path(&path, "2026-03-25 09:16", &mut picker).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [x] 11:32 current task\n- [ ] 11:48 pending task\n"
    );
}

#[test]
fn done_command_interactive_cancel_leaves_log_unchanged() {
    let dir = unique_temp_dir("done-interactive-cancel");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first unfinished\n- [ ] 11:48 second unfinished\n",
    )
    .unwrap();

    let before = fs::read_to_string(&path).unwrap();
    let mut picker = FakeDonePicker::new(None);
    done_command::run_interactive_at_path(&path, "2026-03-24 11:52", &mut picker).unwrap();

    assert_eq!(picker.calls, 1);
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn done_command_interactive_errors_on_invalid_state_vector() {
    let dir = unique_temp_dir("done-interactive-invalid-state-vector");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 only unfinished\n",
    )
    .unwrap();

    let mut picker = FakeDonePicker::new(Some(vec![]));
    let error =
        done_command::run_interactive_at_path(&path, "2026-03-24 11:52", &mut picker).unwrap_err();

    assert!(format!("{error:#}").contains("state"), "{error:#}");
}

#[test]
fn done_store_rejects_stale_unfinished_entry_targets() {
    let dir = unique_temp_dir("done-stale-target");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 only unfinished\n",
    )
    .unwrap();

    let target = store::collect_unfinished_entries(&path)
        .unwrap()
        .pop()
        .unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 only unfinished (renamed)\n",
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
        "# wid log\n\n## 2026-03-24   \n\n- [ ] 11:32 spaced entry   \n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24   \n\n- [x] 11:32 spaced entry   \n"
    );
}

#[test]
fn done_store_collects_unfinished_entries_in_oldest_first_order() {
    let dir = unique_temp_dir("done-collect-order");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first unfinished\n\n## 2026-03-25\n\n- [ ] 09:15 latest unfinished\n",
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
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 11:48 address review feedback\n- [ ] 12:10 rework implementation plan\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:50").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 11:48 address review feedback\n- [x] 12:10 rework implementation plan\n"
    );
}

#[test]
fn done_store_skips_trailing_done_entries() {
    let dir = unique_temp_dir("done-store-skip-trailing");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 11:48 address review feedback\n- [x] 12:10 rework implementation plan\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [x] 11:32 fix failing CI\n- [x] 11:48 address review feedback\n- [x] 12:10 rework implementation plan\n"
    );
}

#[test]
fn done_store_treats_interior_done_marker_as_unfinished() {
    let dir = unique_temp_dir("done-store-interior-marker");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 @done( mentions a topic, but this is still unfinished\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [x] 11:32 @done( mentions a topic, but this is still unfinished\n"
    );
}

#[test]
fn done_store_ignores_later_non_entry_bullet_lines() {
    let dir = unique_temp_dir("done-store-ignore-note");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [ ] 12:10 rework implementation plan\n- note: follow-up detail from the same day\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 12:10 rework implementation plan\n- note: follow-up detail from the same day\n"
    );
}

#[test]
fn done_store_ignores_entry_looking_lines_before_day_section() {
    let dir = unique_temp_dir("done-store-before-section");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 12:10 rework implementation plan\n\n## Notes\n\n- [ ] 11:32 outside any day section\n",
    )
    .unwrap();

    store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [x] 12:10 rework implementation plan\n\n## Notes\n\n- [ ] 11:32 outside any day section\n"
    );
}

#[test]
fn done_store_errors_when_no_unfinished_entry_exists() {
    let dir = unique_temp_dir("done-store-empty");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [x] 11:32 fix failing CI\n",
    )
    .unwrap();

    let error = store::mark_last_unfinished_entry_done(&path, "2026-03-24 11:52").unwrap_err();
    let message = format!("{error:#}");

    assert!(message.contains("unfinished"), "{message}");
}

struct FakeDonePicker {
    result: Option<Vec<model::EntryState>>,
    calls: usize,
    items: Vec<String>,
    default_selected: Option<usize>,
}

impl FakeDonePicker {
    fn new(result: Option<Vec<model::EntryState>>) -> Self {
        Self {
            result,
            calls: 0,
            items: Vec::new(),
            default_selected: None,
        }
    }
}

impl done_picker::DoneStatePicker for FakeDonePicker {
    fn pick_done_states(
        &mut self,
        entries: &[model::LogEntry],
        selected: usize,
    ) -> anyhow::Result<Option<Vec<model::EntryState>>> {
        self.calls += 1;
        self.items = entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                let mut rows = Vec::new();
                if index == 0 || entries[index - 1].date != entry.date {
                    if index > 0 {
                        rows.push(" ".to_string());
                    }
                    let heading = show_command::render_day_heading(&entry.date);
                    rows.push(heading.clone());
                    rows.push("─".repeat(heading.chars().count()));
                }
                rows.extend(entry.display_label().lines().map(|line| {
                    if let Some((_, rest)) = line.split_once(' ')
                        && rest.starts_with("[")
                    {
                        let summary =
                            show_command::render_entry_summary(&entry.summary, &entry.tags);
                        return format!(
                            "{} {}  {}",
                            entry.state.display_marker(),
                            summary,
                            entry.time
                        );
                    }
                    line.to_string()
                }));
                rows
            })
            .collect();
        self.default_selected = Some(selected);
        Ok(self.result.clone())
    }
}
