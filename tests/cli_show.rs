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
mod log {
    pub mod format {
        pub use crate::format::*;
    }
    pub mod model {
        pub use crate::model::*;
    }
    pub mod store {
        pub use crate::store::*;
    }
}
#[path = "../src/commands/show.rs"]
mod show_command;

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
fn wid_without_arguments_prints_all_stored_log_entries() {
    let home = unique_temp_dir("show-default");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 CI が落ちていたので修正\n- [x] 12:10 実装方針を見直した\n",
    )
    .unwrap();

    let output = run_wid(&home, &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "## 2026-03-24\n\n- [ ] 11:32 CI が落ちていたので修正\n- [x] 12:10 実装方針を見直した\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn wid_omits_empty_day_sections_from_output() {
    let home = unique_temp_dir("show-skip-empty-day");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "## 2026-03-24\n\n## 2026-03-25\n\n- [ ] 11:32 CI が落ちていたので修正\n",
    )
    .unwrap();

    let output = run_wid(&home, &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "## 2026-03-25\n\n- [ ] 11:32 CI が落ちていたので修正\n"
    );
}

#[test]
fn render_document_with_color_highlights_active_and_done_entries() {
    let document = parser::parse_log(
        "## 2026-03-24\n\n- [>] 11:32 active\n  - first note\n- [x] 12:10 done\n  - done note\n- [ ] 12:30 pending\n",
    )
    .unwrap();

    let output = show_command::render_document(&document, true);

    assert!(output.contains("\u{1b}["), "{output:?}");
    assert!(output.contains("11:32 active"), "{output:?}");
    assert!(output.contains("  - first note"), "{output:?}");
    assert!(output.contains("12:10 done"), "{output:?}");
    assert!(output.contains("  - done note"), "{output:?}");
    assert!(!output.contains("\n  - done note\n"), "{output:?}");
}
