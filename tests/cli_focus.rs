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
#[path = "../src/commands/focus.rs"]
mod focus_command;

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
            "2026-03-24 11:32 current task".to_string(),
            "2026-03-24 11:48 first pending".to_string(),
            "2026-03-25 09:15 second pending".to_string(),
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
