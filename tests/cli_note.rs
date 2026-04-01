#![allow(dead_code, unused_imports)]

#[path = "../src/interactive/done_picker.rs"]
mod done_picker;
#[allow(dead_code)]
#[path = "../src/log/format.rs"]
mod format;
#[allow(dead_code)]
#[path = "../src/log/model.rs"]
mod model;
#[path = "../src/commands/note.rs"]
mod note_command;
#[path = "../src/commands/now.rs"]
mod now_command;
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
    pub mod paths {
        pub use crate::paths::*;
    }
    pub mod store {
        pub use crate::store::*;
    }
}
mod commands {
    pub mod now {
        pub use crate::now_command::*;
    }
    pub mod show {
        pub use crate::show_command::*;
    }
}
mod interactive {
    pub mod done_picker {
        pub use crate::done_picker::*;
    }
}

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

fn day_heading(date: &str) -> String {
    show_command::render_day_heading(date)
}

fn day_separator(date: &str) -> String {
    let heading = day_heading(date);
    "─".repeat(heading.chars().count())
}

#[test]
fn note_appends_to_active_entry() {
    let dir = unique_temp_dir("note-active");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n- [ ] 08:12 pending item\n",
    )
    .unwrap();

    note_command::run_at_path(&path, vec!["first".into(), "note".into()], None).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:01 active item\n  - first note\n- [ ] 08:12 pending item\n"
    );
}

#[test]
fn note_appends_to_last_pending_when_no_active_exists() {
    let dir = unique_temp_dir("note-pending");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first pending\n- [ ] 08:12 last pending\n",
    )
    .unwrap();

    note_command::run_at_path(&path, vec!["remember".into(), "this".into()], None).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:01 first pending\n- [ ] 08:12 last pending\n  - remember this\n"
    );
}

#[test]
fn note_appends_after_existing_notes() {
    let dir = unique_temp_dir("note-append-existing");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n  - first note\n",
    )
    .unwrap();

    note_command::run_at_path(&path, vec!["second".into(), "note".into()], None).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:01 active item\n  - first note\n  - second note\n"
    );
}

#[test]
fn note_errors_when_no_open_entry_exists() {
    let dir = unique_temp_dir("note-empty");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-25\n\n- [x] 08:01 done item\n").unwrap();

    let error = note_command::run_at_path(&path, vec!["orphan".into()], None).unwrap_err();

    assert!(format!("{error:#}").contains("note target"), "{error:#}");
}

#[test]
fn note_appends_to_entry_by_transient_id() {
    let home = unique_temp_dir("note-by-id");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n- [ ] 08:12 pending item\n",
    )
    .unwrap();

    let json_output = run_wid(&home, &["--json"], None);
    assert!(json_output.status.success(), "{json_output:?}");
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][1]["id"].as_str().unwrap();

    note_command::run_at_path(&log_path, vec!["tracked".into(), "note".into()], Some(id)).unwrap();

    assert_eq!(
        fs::read_to_string(&log_path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:01 active item\n- [ ] 08:12 pending item\n  - tracked note\n"
    );
}

#[test]
fn note_by_id_errors_when_item_changed() {
    let home = unique_temp_dir("note-by-id-stale");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(&log_path, "## 2026-03-25\n\n- [ ] 08:12 pending item\n").unwrap();

    let json_output = run_wid(&home, &["--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    fs::write(
        &log_path,
        "## 2026-03-25\n\n- [ ] 08:12 pending item (edited)\n",
    )
    .unwrap();

    let error =
        note_command::run_at_path(&log_path, vec!["tracked".into(), "note".into()], Some(&id))
            .unwrap_err();

    assert!(
        format!("{error:#}").contains("changed or not found"),
        "{error:#}"
    );
}

#[test]
fn note_reads_single_line_from_stdin_when_no_args_are_given() {
    let home = unique_temp_dir("note-stdin");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(&log_path, "## 2026-03-25\n\n- [>] 08:01 active item\n").unwrap();

    let output = run_wid(&home, &["note"], Some("capture stdin note text\n"));

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(&log_path).unwrap();
    assert!(
        contents.contains("  - capture stdin note text"),
        "{contents}"
    );
}

#[test]
fn note_command_interactive_appends_to_selected_entry() {
    let dir = unique_temp_dir("note-interactive");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n  - existing active note\n- [x] 08:06 done item\n  - existing done note\n- [ ] 08:12 pending item\n  - existing pending note\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(2));
    let mut editor = FakeSummaryEditor::new("selected note");
    note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:01 active item\n  - existing active note\n- [x] 08:06 done item\n  - existing done note\n- [ ] 08:12 pending item\n  - existing pending note\n  - selected note\n"
    );
    assert_eq!(
        picker.rows,
        vec![
            day_heading("2026-03-25"),
            day_separator("2026-03-25"),
            "◉ active item  08:01".to_string(),
            "  · existing active note".to_string(),
            "☑ done item  08:06".to_string(),
            "  · existing done note".to_string(),
            "□ pending item  08:12".to_string(),
            "  · existing pending note".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn note_command_interactive_cancels_without_changes() {
    let dir = unique_temp_dir("note-interactive-cancel");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-25\n\n- [ ] 08:12 pending item\n").unwrap();

    let before = fs::read_to_string(&path).unwrap();
    let mut picker = FakePicker::new(None);
    let mut editor = FakeSummaryEditor::new("unused");
    note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn note_command_interactive_errors_when_there_is_no_entry() {
    let dir = unique_temp_dir("note-interactive-no-entry");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "").unwrap();

    let mut picker = FakePicker::new(None);
    let mut editor = FakeSummaryEditor::new("unused");
    let error = note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap_err();

    assert!(format!("{error:#}").contains("entry"), "{error:#}");
}

#[test]
fn note_command_interactive_defaults_to_first_item_when_no_active_exists() {
    let dir = unique_temp_dir("note-interactive-default");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first pending\n- [ ] 08:12 second pending\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(None);
    let mut editor = FakeSummaryEditor::new("unused");
    note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap();

    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn note_command_interactive_starts_at_top_even_when_an_active_item_exists() {
    let dir = unique_temp_dir("note-interactive-active-not-default");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first pending\n  - first pending note\n- [>] 08:12 active item\n  - active note\n- [ ] 08:20 later pending\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(None);
    let mut editor = FakeSummaryEditor::new("unused");
    note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap();

    assert_eq!(
        picker.rows,
        vec![
            day_heading("2026-03-25"),
            day_separator("2026-03-25"),
            "□ first pending  08:01".to_string(),
            "  · first pending note".to_string(),
            "◉ active item  08:12".to_string(),
            "  · active note".to_string(),
            "□ later pending  08:20".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn note_command_interactive_shows_done_items_but_defaults_to_latest_open_item() {
    let dir = unique_temp_dir("note-interactive-done-visible");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first pending\n  - first pending note\n- [x] 08:12 done item\n  - done note\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(None);
    let mut editor = FakeSummaryEditor::new("unused");
    note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap();

    assert_eq!(
        picker.rows,
        vec![
            day_heading("2026-03-25"),
            day_separator("2026-03-25"),
            "□ first pending  08:01".to_string(),
            "  · first pending note".to_string(),
            "☑ done item  08:12".to_string(),
            "  · done note".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn note_command_interactive_inserts_blank_line_before_later_date_header() {
    let dir = unique_temp_dir("note-interactive-date-gap");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 older pending\n  - older note\n\n## 2026-03-26\n\n- [ ] 09:12 newer pending\n  - newer note\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(None);
    let mut editor = FakeSummaryEditor::new("unused");
    note_command::run_interactive_at_path(&path, &mut picker, &mut editor).unwrap();

    assert_eq!(
        picker.rows,
        vec![
            day_heading("2026-03-25"),
            day_separator("2026-03-25"),
            "□ older pending  08:01".to_string(),
            "  · older note".to_string(),
            " ".to_string(),
            day_heading("2026-03-26"),
            day_separator("2026-03-26"),
            "□ newer pending  09:12".to_string(),
            "  · newer note".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn note_by_id_reads_single_line_from_stdin_when_no_args_are_given() {
    let home = unique_temp_dir("note-by-id-stdin");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n- [ ] 08:12 pending item\n",
    )
    .unwrap();

    let json_output = run_wid(&home, &["--json"], None);
    assert!(json_output.status.success(), "{json_output:?}");
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][1]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(
        &home,
        &["note", "--id", &id],
        Some("stdin note for targeted entry\n"),
    );

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(&log_path).unwrap();
    assert!(
        contents.contains("  - stdin note for targeted entry"),
        "{contents}"
    );
}

#[test]
fn note_rejects_duplicate_text_for_same_item() {
    let dir = unique_temp_dir("note-duplicate");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n  - existing note\n",
    )
    .unwrap();

    let error =
        note_command::run_at_path(&path, vec!["existing".into(), "note".into()], None).unwrap_err();

    assert!(
        format!("{error:#}").contains("duplicate note text for item"),
        "{error:#}"
    );
}

struct FakePicker {
    result: Option<usize>,
    rows: Vec<String>,
    default_selected: Option<usize>,
}

impl FakePicker {
    fn new(result: Option<usize>) -> Self {
        Self {
            result,
            rows: Vec::new(),
            default_selected: None,
        }
    }
}

impl done_picker::GroupedLogEntryPicker for FakePicker {
    fn pick_grouped_entries(
        &mut self,
        entries: &[model::LogEntry],
        selected: usize,
    ) -> anyhow::Result<Option<usize>> {
        self.rows.clear();
        let mut current_date: Option<&str> = None;
        for entry in entries {
            if current_date != Some(entry.date.as_str()) {
                if !self.rows.is_empty() {
                    self.rows.push(" ".to_string());
                }
                let heading = show_command::render_day_heading(&entry.date);
                self.rows.push(heading.clone());
                self.rows.push("─".repeat(heading.chars().count()));
                current_date = Some(entry.date.as_str());
            }

            self.rows.push(format!(
                "{} {}  {}",
                entry.state.display_marker(),
                show_command::render_entry_summary(&entry.summary, &entry.tags),
                entry.time
            ));
            self.rows.extend(
                entry
                    .notes
                    .iter()
                    .map(|note| model::format_note_display(note)),
            );
        }
        self.default_selected = Some(selected);
        Ok(self.result)
    }
}

struct FakeSummaryEditor {
    text: String,
}

impl FakeSummaryEditor {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl note_command::NoteEditor for FakeSummaryEditor {
    fn edit_note(&mut self) -> anyhow::Result<String> {
        Ok(self.text.clone())
    }
}
