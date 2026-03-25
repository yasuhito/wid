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
#[path = "../src/commands/show.rs"]
mod show_command;
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
#[path = "../src/commands/focus.rs"]
mod focus_command;

use std::fs;
use std::process::Command;
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

fn wid_bin() -> &'static str {
    env!("CARGO_BIN_EXE_wid")
}

fn run_wid(home: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(wid_bin())
        .env("HOME", home)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn focus_interactive_promotes_selected_entry_and_clears_previous_active() {
    let dir = unique_temp_dir("focus-select");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 selected task\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(1));
    focus_command::run_interactive_at_path(&path, &mut picker).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 current task\n- [>] 11:48 selected task\n"
    );
}

#[test]
fn focus_interactive_lists_only_pending_entries() {
    let dir = unique_temp_dir("focus-order");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 first pending\n- [x] 12:10 done\n\n## 2026-03-25\n\n- [ ] 09:15 second pending\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(None);
    focus_command::run_interactive_at_path(&path, &mut picker).unwrap();

    assert_eq!(
        picker.items,
        vec![
            "2026-03-24 [>] 11:32 current task".to_string(),
            "2026-03-24 [ ] 11:48 first pending".to_string(),
            "2026-03-25 [ ] 09:15 second pending".to_string(),
        ]
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn focus_interactive_errors_when_no_focusable_entry_exists() {
    let dir = unique_temp_dir("focus-empty");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [x] 11:48 done\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(0));
    let error = focus_command::run_interactive_at_path(&path, &mut picker).unwrap_err();
    assert!(format!("{error:#}").contains("pending"), "{error:#}");
}

#[test]
fn focus_interactive_keeps_active_item_unchanged_when_reselected() {
    let dir = unique_temp_dir("focus-active");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 selected task\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(0));
    focus_command::run_interactive_at_path(&path, &mut picker).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 selected task\n"
    );
    assert_eq!(picker.default_selected, Some(0));
}

#[test]
fn focus_interactive_updates_checkbox_not_summary_marker() {
    let dir = unique_temp_dir("focus-summary-marker");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-25\n\n- [>] 07:39 README.md を追加\n- [ ] 08:01 rm -i でアイテムの [ ] と [x], [>] も表示するようにする。\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(1));
    focus_command::run_interactive_at_path(&path, &mut picker).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-25\n\n- [ ] 07:39 README.md を追加\n- [>] 08:01 rm -i でアイテムの [ ] と [x], [>] も表示するようにする。\n"
    );
}

#[test]
fn focus_defaults_to_latest_entry_and_clears_previous_active() {
    let dir = unique_temp_dir("focus-default-latest");
    let home = dir;
    let path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- [>] 11:32 current task\n- [ ] 11:48 latest task\n",
    )
    .unwrap();

    let output = run_wid(&home, &["focus"]);

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- [>] 11:48 latest task"), "{stdout}");

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 current task\n- [>] 11:48 latest task\n"
    );
}

#[test]
fn focus_defaults_is_noop_when_latest_entry_is_already_active() {
    let dir = unique_temp_dir("focus-default-active");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first task\n- [>] 11:48 latest task\n").unwrap();

    focus_command::run_at_path(&path, false).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first task\n- [>] 11:48 latest task\n"
    );
}

#[test]
fn focus_defaults_errors_when_latest_entry_is_done() {
    let dir = unique_temp_dir("focus-default-done");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 first task\n- [x] 11:48 latest task\n").unwrap();

    let error = focus_command::run_at_path(&path, false).unwrap_err();

    assert!(format!("{error:#}").contains("done"), "{error:#}");
}

#[test]
fn focus_defaults_errors_when_no_entry_exists() {
    let dir = unique_temp_dir("focus-default-empty");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-24\n\n").unwrap();

    let error = focus_command::run_at_path(&path, false).unwrap_err();

    assert!(format!("{error:#}").contains("entry"), "{error:#}");
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
    fn pick<T: model::PickerItem>(&mut self, entries: &[T]) -> anyhow::Result<Option<usize>> {
        self.pick_with_selected(entries, 0)
    }

    fn pick_with_selected<T: model::PickerItem>(
        &mut self,
        entries: &[T],
        selected: usize,
    ) -> anyhow::Result<Option<usize>> {
        self.items = entries.iter().map(model::PickerItem::display_label).collect();
        self.default_selected = Some(selected);
        Ok(self.result)
    }
}
