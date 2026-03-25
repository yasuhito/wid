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
#[path = "../src/commands/rm.rs"]
mod rm_command;

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
fn rm_interactive_lists_entries_and_notes_in_wid_order() {
    let dir = unique_temp_dir("rm-list-order");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-24\n\n- [ ] 11:32 first unfinished\n  - first note\n- [x] 12:10 already done\n\n## 2026-03-25\n\n- [ ] 09:15 newest item\n  - newest note\n",
    )
    .unwrap();

    let entries = store::collect_removable_targets(&path).unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.display_label())
            .collect::<Vec<_>>(),
        vec![
            "2026-03-24 [ ] 11:32 first unfinished".to_string(),
            "  📝 first note".to_string(),
            "2026-03-24 [x] 12:10 already done".to_string(),
            "2026-03-25 [ ] 09:15 newest item".to_string(),
            "  📝 newest note".to_string(),
        ]
    );
}

#[test]
fn rm_interactive_keeps_completed_entries_visible() {
    let dir = unique_temp_dir("rm-non-timestamp-done-marker");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-25\n\n- [x] 09:15 investigate\n",
    )
    .unwrap();

    let entries = store::collect_removable_targets(&path).unwrap();
    assert_eq!(
        entries[0].display_label(),
        "2026-03-25 [x] 09:15 investigate"
    );
}

#[test]
fn rm_command_interactive_deletes_selected_entry_after_confirmation() {
    let dir = unique_temp_dir("rm-delete-selected");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-24\n\n- [ ] 11:32 first unfinished\n- [x] 11:48 already done\n\n## 2026-03-25\n\n- [ ] 09:15 selected item\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(2));
    let mut confirmer = FakeConfirm::yes();
    rm_command::run_interactive_at_path(&path, &mut picker, &mut confirmer).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:32 first unfinished\n- [x] 11:48 already done\n"
    );
}

#[test]
fn rm_command_interactive_deletes_selected_note_after_confirmation() {
    let dir = unique_temp_dir("rm-delete-selected-note");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-24\n\n- [ ] 11:32 first unfinished\n  - first note\n  - second note\n",
    )
    .unwrap();

    let mut picker = FakePicker::new(Some(1));
    let mut confirmer = FakeConfirm::yes();
    rm_command::run_interactive_at_path(&path, &mut picker, &mut confirmer).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:32 first unfinished\n  - second note\n"
    );
}

#[test]
fn rm_command_interactive_does_not_delete_when_confirmation_is_not_yes() {
    let dir = unique_temp_dir("rm-cancel-confirm");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "## 2026-03-24\n\n- [ ] 11:32 only item\n").unwrap();

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
    fs::write(&path, "## 2026-03-24\n\n- [ ] 11:32 selected item\n").unwrap();

    let target = store::collect_entries(&path).unwrap().pop().unwrap();
    fs::write(
        &path,
        "## 2026-03-24\n\n- [ ] 11:32 selected item (renamed)\n",
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
    let error =
        rm_command::run_interactive_at_path(&path, &mut picker, &mut confirmer).unwrap_err();
    assert!(format!("{error:#}").contains("no entries"), "{error:#}");
}

#[test]
fn rm_command_by_id_deletes_entry() {
    let home = unique_temp_dir("rm-by-id-entry");
    write_log(
        &home,
        "## 2026-03-25\n\n- [ ] 08:01 first item\n- [ ] 08:12 target item\n",
    );

    let json_output = run_wid(&home, &["--json"], None);
    assert!(json_output.status.success(), "{json_output:?}");
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][1]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["rm", "--id", &id], None);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(log_path(&home)).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:01 first item\n"
    );
}

#[test]
fn rm_command_by_id_deletes_note() {
    let home = unique_temp_dir("rm-by-id-note");
    write_log(
        &home,
        "## 2026-03-25\n\n- [ ] 08:12 target item\n  - first note\n  - second note\n",
    );

    let json_output = run_wid(&home, &["--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["notes"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["rm", "--id", &id], None);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(log_path(&home)).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:12 target item\n  - second note\n"
    );
}

#[test]
fn rm_command_by_id_errors_when_item_changed() {
    let home = unique_temp_dir("rm-by-id-stale");
    write_log(&home, "## 2026-03-25\n\n- [ ] 08:12 target item\n");

    let json_output = run_wid(&home, &["--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    write_log(&home, "## 2026-03-25\n\n- [ ] 08:12 target item (edited)\n");

    let output = run_wid(&home, &["rm", "--id", &id], None);

    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("changed or not found"), "{stderr}");
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
