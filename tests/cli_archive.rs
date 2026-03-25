#![allow(dead_code, unused_imports)]

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
#[path = "../src/commands/show.rs"]
mod show_command;
mod log {
    pub mod model {
        pub use crate::model::*;
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

fn log_path(home: &PathBuf) -> PathBuf {
    home.join(".local/share/wid/log.md")
}

fn archive_path(home: &PathBuf) -> PathBuf {
    home.join(".local/share/wid/archive.md")
}

#[test]
fn archive_moves_all_done_entries_and_keeps_open_entries_in_log() {
    let home = unique_temp_dir("archive-move-done");
    let log = log_path(&home);
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    fs::write(
        &log,
        "## 2026-03-24\n\n- [x] 11:32 finished task\n  - done note\n- [ ] 11:48 pending task\n\n## 2026-03-25\n\n- [x] 09:15 another done task\n- [>] 09:30 active task\n",
    )
    .unwrap();

    let output = run_wid(&home, &["archive"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:48 pending task\n\n## 2026-03-25\n\n- [>] 09:30 active task\n"
    );
    assert_eq!(
        fs::read_to_string(archive_path(&home)).unwrap(),
        "## 2026-03-24\n\n- [x] 11:32 finished task\n  - done note\n\n## 2026-03-25\n\n- [x] 09:15 another done task\n"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- [ ] 11:48 pending task"), "{stdout}");
    assert!(stdout.contains("- [>] 09:30 active task"), "{stdout}");
    assert!(!stdout.contains("finished task"), "{stdout}");
}

#[test]
fn archive_reuses_existing_archive_day_sections() {
    let home = unique_temp_dir("archive-reuse-day");
    let log = log_path(&home);
    let archive = archive_path(&home);
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    fs::write(
        &log,
        "## 2026-03-24\n\n- [x] 11:32 finished task\n",
    )
    .unwrap();
    fs::write(
        &archive,
        "## 2026-03-24\n\n- [x] 09:10 archived earlier\n",
    )
    .unwrap();

    let output = run_wid(&home, &["archive"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&archive).unwrap(),
        "## 2026-03-24\n\n- [x] 09:10 archived earlier\n- [x] 11:32 finished task\n"
    );
}

#[test]
fn archive_leaves_files_clean_when_there_are_no_done_entries() {
    let home = unique_temp_dir("archive-no-done");
    let log = log_path(&home);
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    fs::write(
        &log,
        "## 2026-03-24\n\n- [ ] 11:48 pending task\n- [>] 12:10 active task\n",
    )
    .unwrap();

    let output = run_wid(&home, &["archive"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "## 2026-03-24\n\n- [ ] 11:48 pending task\n- [>] 12:10 active task\n"
    );
    assert!(!archive_path(&home).exists());
}
