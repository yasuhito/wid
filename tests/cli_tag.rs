#![allow(dead_code, unused_imports)]

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
mod log {
    pub mod format {
        pub use crate::format::*;
    }
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
    pub mod show {
        pub use crate::show_command::*;
    }
}
#[path = "../src/commands/tag.rs"]
mod tag_command;

use std::fs;
use std::path::{Path, PathBuf};
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

fn log_path(home: &Path) -> PathBuf {
    home.join(".local/share/wid/log.md")
}

fn write_log(home: &Path, contents: &str) {
    let path = log_path(home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn tag_add_by_id_appends_tags_without_duplicates() {
    let home = unique_temp_dir("tag-add");
    write_log(&home, "## 2026-03-25\n\n- [ ] 08:12 pending item @wid\n");

    let json_output = run_wid(&home, &["--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["tag", "add", "--id", &id, "@agent", "@wid"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(log_path(&home)).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:12 pending item @wid @agent\n"
    );
}

#[test]
fn tag_rm_by_id_removes_requested_tags() {
    let home = unique_temp_dir("tag-rm");
    write_log(
        &home,
        "## 2026-03-25\n\n- [ ] 08:12 pending item @wid @agent @ci\n",
    );

    let json_output = run_wid(&home, &["--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["tag", "rm", "--id", &id, "@agent"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(log_path(&home)).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:12 pending item @wid @ci\n"
    );
}

#[test]
fn tag_rm_missing_tag_is_noop() {
    let home = unique_temp_dir("tag-rm-missing");
    write_log(&home, "## 2026-03-25\n\n- [ ] 08:12 pending item @wid\n");

    let json_output = run_wid(&home, &["--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["tag", "rm", "--id", &id, "@agent"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(log_path(&home)).unwrap(),
        "## 2026-03-25\n\n- [ ] 08:12 pending item @wid\n"
    );
}

#[test]
fn tag_add_by_id_rejects_note_ids() {
    let home = unique_temp_dir("tag-note-id");
    write_log(
        &home,
        "## 2026-03-25\n\n- [ ] 08:12 pending item\n  - first note\n",
    );

    let json_output = run_wid(&home, &["--json"]);
    let value: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let id = value["days"][0]["entries"][0]["notes"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let output = run_wid(&home, &["tag", "add", "--id", &id, "@wid"]);

    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("item changed or not found"),
        "{output:?}"
    );
}
