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
#[path = "../src/commands/edit.rs"]
mod edit_command;

use std::fs;
use std::path::PathBuf;
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

#[test]
fn edit_updates_active_entry_summary_by_default() {
    let dir = unique_temp_dir("edit-active");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 backlog item\n- [>] 08:12 active item\n  - keep note\n",
    )
    .unwrap();

    let mut editor = FakeEditor::with_response(Some("renamed active item"));
    edit_command::run_at_path(&path, false, &mut NoopPicker, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:01 backlog item\n- [>] 08:12 renamed active item\n  - keep note\n"
    );
    assert_eq!(editor.initial_summary.as_deref(), Some("active item"));
}

#[test]
fn edit_updates_latest_entry_when_no_active_exists() {
    let dir = unique_temp_dir("edit-latest");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first item\n- [x] 08:12 latest item\n",
    )
    .unwrap();

    let mut editor = FakeEditor::with_response(Some("renamed latest item"));
    edit_command::run_at_path(&path, false, &mut NoopPicker, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:01 first item\n- [x] 08:12 renamed latest item\n"
    );
    assert_eq!(editor.initial_summary.as_deref(), Some("latest item"));
}

#[test]
fn edit_interactive_updates_selected_entry_only() {
    let dir = unique_temp_dir("edit-interactive");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first item\n- [>] 08:12 active item\n- [x] 08:30 done item\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(2));
    let mut editor = FakeEditor::with_response(Some("renamed done item"));
    edit_command::run_at_path(&path, true, &mut picker, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:01 first item\n- [>] 08:12 active item\n- [x] 08:30 renamed done item\n"
    );
    assert_eq!(
        picker.items,
        vec![
            "Yesterday · 2026-03-25 Wed".to_string(),
            "─".repeat("Yesterday · 2026-03-25 Wed".chars().count()),
            "□ first item  08:01".to_string(),
            "◉ active item  08:12".to_string(),
            "☑ done item  08:30".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(1));
    assert_eq!(editor.initial_summary.as_deref(), Some("done item"));
}

#[test]
fn edit_interactive_updates_selected_note_only() {
    let dir = unique_temp_dir("edit-interactive-note");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:12 active item\n  - first note\n  - second note\n- [ ] 08:30 later item\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(2));
    let mut editor = FakeEditor::with_response(Some("edited second note"));
    edit_command::run_at_path(&path, true, &mut picker, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:12 active item\n  - first note\n  - edited second note\n- [ ] 08:30 later item\n"
    );
    assert_eq!(
        picker.items,
        vec![
            "Yesterday · 2026-03-25 Wed".to_string(),
            "─".repeat("Yesterday · 2026-03-25 Wed".chars().count()),
            "◉ active item  08:12".to_string(),
            "  · first note".to_string(),
            "  · second note".to_string(),
            "□ later item  08:30".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
    assert_eq!(editor.initial_summary.as_deref(), Some("second note"));
}

#[test]
fn edit_interactive_defaults_to_latest_entry_when_no_active_exists() {
    let dir = unique_temp_dir("edit-interactive-default-latest");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [ ] 08:01 first item\n- [x] 08:30 latest item\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(None);
    let mut editor = FakeEditor::with_response(None);
    edit_command::run_at_path(&path, true, &mut picker, &mut editor).unwrap();

    assert_eq!(picker.default_selected, Some(1));
}

#[test]
fn edit_rejects_empty_summary() {
    let dir = unique_temp_dir("edit-empty");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-25\n\n- [>] 08:12 active item\n").unwrap();

    let mut editor = FakeEditor::with_response(Some("   "));
    let error = edit_command::run_at_path(&path, false, &mut NoopPicker, &mut editor).unwrap_err();

    assert!(format!("{error:#}").contains("empty summary"), "{error:#}");
}

#[test]
fn edit_errors_when_no_entry_exists() {
    let dir = unique_temp_dir("edit-none");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-25\n\n").unwrap();

    let mut editor = FakeEditor::with_response(Some("unused"));
    let error = edit_command::run_at_path(&path, false, &mut NoopPicker, &mut editor).unwrap_err();

    assert!(format!("{error:#}").contains("no entry"), "{error:#}");
}

#[test]
fn edit_keeps_checkbox_markers_inside_summary_text() {
    let dir = unique_temp_dir("edit-summary-marker");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:12 investigate [>] marker\n",
    )
    .unwrap();

    let mut editor = FakeEditor::with_response(Some("edited [>] marker [x] text"));
    edit_command::run_at_path(&path, false, &mut NoopPicker, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:12 edited [>] marker [x] text\n"
    );
}

#[test]
fn edit_updates_entry_by_transient_id() {
    let dir = unique_temp_dir("edit-by-entry-id");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:12 active item @wid\n  - keep note\n",
    )
    .unwrap();

    let document = parser::parse_log(&fs::read_to_string(&path).unwrap()).unwrap();
    let entry = &document.days[0].entries[0];
    let id = entry.transient_id("2026-03-25");

    let mut editor = FakeEditor::with_response(Some("renamed active item @wid"));
    edit_command::run_by_id_at_path(&path, &id, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:12 renamed active item @wid\n  - keep note\n"
    );
    assert_eq!(editor.initial_summary.as_deref(), Some("active item @wid"));
}

#[test]
fn edit_updates_note_by_transient_id() {
    let dir = unique_temp_dir("edit-by-note-id");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:12 active item\n  - first note\n  - second note\n",
    )
    .unwrap();

    let document = parser::parse_log(&fs::read_to_string(&path).unwrap()).unwrap();
    let entry = &document.days[0].entries[0];
    let id = entry.transient_note_id("2026-03-25", "second note");

    let mut editor = FakeEditor::with_response(Some("edited second note"));
    edit_command::run_by_id_at_path(&path, &id, &mut editor).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-25\n\n- [>] 08:12 active item\n  - first note\n  - edited second note\n"
    );
    assert_eq!(editor.initial_summary.as_deref(), Some("second note"));
}

#[test]
fn edit_by_id_errors_when_target_changed() {
    let dir = unique_temp_dir("edit-by-id-stale");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-25\n\n- [ ] 08:12 pending item\n").unwrap();

    let document = parser::parse_log(&fs::read_to_string(&path).unwrap()).unwrap();
    let entry = &document.days[0].entries[0];
    let id = entry.transient_id("2026-03-25");

    fs::write(&path, "## 2026-03-25\n\n- [ ] 08:12 pending item edited\n").unwrap();

    let mut editor = FakeEditor::with_response(Some("new text"));
    let error = edit_command::run_by_id_at_path(&path, &id, &mut editor).unwrap_err();

    assert!(
        format!("{error:#}").contains("item changed or not found"),
        "{error:#}"
    );
}

struct FakePicker {
    result: Option<usize>,
    items: Vec<String>,
    default_selected: Option<usize>,
}

impl FakePicker {
    fn new(result: Option<usize>) -> Self {
        Self {
            result,
            items: Vec::new(),
            default_selected: None,
        }
    }
}

impl done_picker::Picker for FakePicker {
    fn pick<T: model::PickerItem + model::GroupedPickerItem>(
        &mut self,
        entries: &[T],
    ) -> anyhow::Result<Option<usize>> {
        self.pick_with_selected(entries, 0)
    }

    fn pick_with_selected<T: model::PickerItem + model::GroupedPickerItem>(
        &mut self,
        entries: &[T],
        selected: usize,
    ) -> anyhow::Result<Option<usize>> {
        self.items = entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                let mut rows = Vec::new();
                if index == 0 || entries[index - 1].group_date() != entry.group_date() {
                    if index > 0 {
                        rows.push(" ".to_string());
                    }
                    let heading = show_command::render_day_heading(entry.group_date());
                    rows.push(heading.clone());
                    rows.push("─".repeat(heading.chars().count()));
                }
                rows.extend(entry.grouped_display_label().lines().map(str::to_string));
                rows
            })
            .collect();
        self.default_selected = Some(selected);
        Ok(self.result)
    }
}

struct NoopPicker;

impl done_picker::Picker for NoopPicker {
    fn pick<T: model::PickerItem + model::GroupedPickerItem>(
        &mut self,
        _entries: &[T],
    ) -> anyhow::Result<Option<usize>> {
        panic!("picker should not be used")
    }
}

#[derive(Default)]
struct FakeEditor {
    response: Option<&'static str>,
    initial_summary: Option<String>,
}

impl FakeEditor {
    fn with_response(response: Option<&'static str>) -> Self {
        Self {
            response,
            initial_summary: None,
        }
    }
}

impl edit_command::SummaryEditor for FakeEditor {
    fn edit_summary(&mut self, initial: &str) -> anyhow::Result<String> {
        self.initial_summary = Some(initial.to_string());
        Ok(self.response.unwrap_or(initial).to_string())
    }
}
