#![allow(dead_code, unused_imports)]

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

use std::fs;
use std::path::PathBuf;
use std::process::Command;
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

    let json_output = run_wid(&home, &["--json"]);
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

    let json_output = run_wid(&home, &["--json"]);
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
