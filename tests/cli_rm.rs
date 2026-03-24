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
#[path = "../src/commands/rm.rs"]
mod rm_command;

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
fn rm_interactive_lists_all_entries_newest_first() {
    let dir = unique_temp_dir("rm-list-order");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n- 12:10 already done @done(2026-03-24 12:11)\n\n## 2026-03-25\n\n- 09:15 newest item\n",
    )
    .unwrap();

    let entries = store::collect_entries(&path).unwrap();
    let display_entries: Vec<_> = entries.into_iter().rev().collect();

    assert_eq!(
        display_entries
            .iter()
            .map(|entry| entry.display_label())
            .collect::<Vec<_>>(),
        vec![
            "2026-03-25 09:15 newest item".to_string(),
            "2026-03-24 12:10 already done".to_string(),
            "2026-03-24 11:32 first unfinished".to_string(),
        ]
    );
}

#[test]
fn rm_interactive_keeps_non_timestamp_done_markers_visible() {
    let dir = unique_temp_dir("rm-non-timestamp-done-marker");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-25\n\n- 09:15 investigate @done(tbd)\n",
    )
    .unwrap();

    let entries = store::collect_entries(&path).unwrap();
    let display_entries: Vec<_> = entries.into_iter().rev().collect();

    assert_eq!(
        display_entries[0].display_label(),
        "2026-03-25 09:15 investigate @done(tbd)"
    );
}

#[test]
fn rm_command_interactive_deletes_selected_entry_after_confirmation() {
    let dir = unique_temp_dir("rm-delete-selected");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n- 11:48 already done @done(2026-03-24 11:50)\n\n## 2026-03-25\n\n- 09:15 selected item\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(0));
    let mut confirmer = FakeConfirm::yes();
    rm_command::run_interactive_at_path(&path, &mut picker, &mut confirmer).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 first unfinished\n- 11:48 already done @done(2026-03-24 11:50)\n\n## 2026-03-25\n\n"
    );
}

#[test]
fn rm_command_interactive_does_not_delete_when_confirmation_is_not_yes() {
    let dir = unique_temp_dir("rm-cancel-confirm");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 only item\n",
    )
    .unwrap();

    let before = fs::read_to_string(&path).unwrap();
    let mut picker = FakePicker::new(Some(0));
    let mut confirmer = FakeConfirm::no();
    rm_command::run_interactive_at_path(&path, &mut picker, &mut confirmer).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn rm_store_rejects_stale_entry_targets() {
    let dir = unique_temp_dir("rm-stale-target");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 selected item\n",
    )
    .unwrap();

    let target = store::collect_entries(&path).unwrap().pop().unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-24\n\n- 11:32 selected item (renamed)\n",
    )
    .unwrap();

    let error = store::delete_entry(&path, &target).unwrap_err();
    assert!(format!("{error:#}").contains("changed"), "{error:#}");
}

#[test]
fn rm_store_errors_when_log_has_no_entries() {
    let dir = unique_temp_dir("rm-empty-log");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "# wid log\n").unwrap();

    let mut picker = FakePicker::new(Some(0));
    let mut confirmer = FakeConfirm::yes();
    let error = rm_command::run_interactive_at_path(&path, &mut picker, &mut confirmer).unwrap_err();
    assert!(format!("{error:#}").contains("no entries"), "{error:#}");
}

struct FakePicker {
    result: Option<usize>,
}

impl FakePicker {
    fn new(result: Option<usize>) -> Self {
        Self { result }
    }
}

impl done_picker::Picker for FakePicker {
    fn pick<T: model::PickerItem>(&mut self, _entries: &[T]) -> anyhow::Result<Option<usize>> {
        Ok(self.result)
    }
}

struct FakeConfirm {
    result: bool,
}

impl FakeConfirm {
    fn yes() -> Self {
        Self { result: true }
    }

    fn no() -> Self {
        Self { result: false }
    }
}

impl rm_command::Confirm for FakeConfirm {
    fn confirm(&mut self) -> anyhow::Result<bool> {
        Ok(self.result)
    }
}
