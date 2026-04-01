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

fn day_heading(date: &str) -> String {
    show_command::render_day_heading(date)
}

#[test]
fn wid_without_arguments_prints_all_stored_log_entries() {
    let home = unique_temp_dir("show-default");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32 fix failing CI\n- [x] 12:10 rework implementation plan\n",
    )
    .unwrap();

    let output = run_wid(&home, &[]);

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines[0], "2026-03-24 Tue");
    assert_eq!(lines[1], "─".repeat(lines[0].chars().count()));
    assert_eq!(lines[2], "□ fix failing CI  11:32");
    assert_eq!(lines[3], "☑ rework implementation plan  12:10");
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn wid_omits_empty_day_sections_from_output() {
    let home = unique_temp_dir("show-skip-empty-day");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "## 2026-03-24\n\n## 2026-03-25\n\n- [ ] 11:32 fix failing CI\n",
    )
    .unwrap();

    let output = run_wid(&home, &[]);

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines[0], day_heading("2026-03-25"));
    assert_eq!(lines[1], "─".repeat(lines[0].chars().count()));
    assert_eq!(lines[2], "□ fix failing CI  11:32");
}

#[test]
fn wid_render_uses_today_and_yesterday_labels_for_recent_days() {
    let document = parser::parse_log(
        "## 2026-03-25\n\n- [x] 17:25 yesterday item\n\n## 2026-03-26\n\n- [>] 08:56 today item\n",
    )
    .unwrap();

    let output = show_command::render_document(&document, false);

    assert!(output.contains(&day_heading("2026-03-25")), "{output:?}");
    assert!(output.contains(&day_heading("2026-03-26")), "{output:?}");
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines[1], "─".repeat(lines[0].chars().count()));
    assert_eq!(lines[5], "─".repeat(lines[4].chars().count()));
}

#[test]
fn wid_json_outputs_days_entries_and_transient_ids() {
    let home = unique_temp_dir("show-json");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "## 2026-03-24\n\n- [>] 11:32 active item\n  - first note\n- [x] 12:10 done item\n",
    )
    .unwrap();

    let output = run_wid(&home, &["--json"]);

    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let days = value["days"].as_array().unwrap();
    assert_eq!(days.len(), 1);
    assert_eq!(days[0]["date"], "2026-03-24");
    let entries = days[0]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["state"], "active");
    assert_eq!(entries[0]["time"], "11:32");
    assert_eq!(entries[0]["summary"], "active item");
    assert_eq!(entries[0]["tags"], serde_json::json!([]));
    assert_eq!(entries[0]["notes"][0]["text"], "first note");
    assert!(
        entries[0]["notes"][0]["id"]
            .as_str()
            .unwrap()
            .starts_with("note_")
    );
    assert!(entries[0]["id"].as_str().unwrap().len() >= 12);
    assert_eq!(entries[1]["state"], "done");
}

#[test]
fn wid_json_splits_summary_and_tags() {
    let home = unique_temp_dir("show-json-tags");
    let log_path = home.join(".local/share/wid/log.md");
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(
        &log_path,
        "## 2026-03-24\n\n- [>] 11:32 active item @wid @agent\n",
    )
    .unwrap();

    let output = run_wid(&home, &["--json"]);

    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let entry = &value["days"][0]["entries"][0];
    assert_eq!(entry["summary"], "active item");
    assert_eq!(entry["tags"], serde_json::json!(["wid", "agent"]));
}

#[test]
fn render_json_document_keeps_transient_id_when_notes_change() {
    let before =
        parser::parse_log("## 2026-03-24\n\n- [ ] 11:32 same summary\n  - first note\n").unwrap();
    let after =
        parser::parse_log("## 2026-03-24\n\n- [ ] 11:32 same summary\n  - second note\n").unwrap();

    let before_json: serde_json::Value =
        serde_json::from_str(&show_command::render_document_json(&before)).unwrap();
    let after_json: serde_json::Value =
        serde_json::from_str(&show_command::render_document_json(&after)).unwrap();

    assert_eq!(
        before_json["days"][0]["entries"][0]["id"],
        after_json["days"][0]["entries"][0]["id"]
    );
    assert_ne!(
        before_json["days"][0]["entries"][0]["notes"][0]["id"],
        after_json["days"][0]["entries"][0]["notes"][0]["id"]
    );
}

#[test]
fn render_json_document_keeps_remaining_note_id_when_earlier_note_is_removed() {
    let before = parser::parse_log(
        "## 2026-03-24\n\n- [ ] 11:32 same summary\n  - first note\n  - second note\n",
    )
    .unwrap();
    let after =
        parser::parse_log("## 2026-03-24\n\n- [ ] 11:32 same summary\n  - second note\n").unwrap();

    let before_json: serde_json::Value =
        serde_json::from_str(&show_command::render_document_json(&before)).unwrap();
    let after_json: serde_json::Value =
        serde_json::from_str(&show_command::render_document_json(&after)).unwrap();

    assert_eq!(
        before_json["days"][0]["entries"][0]["notes"][1]["id"],
        after_json["days"][0]["entries"][0]["notes"][0]["id"]
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
    assert!(output.contains("◉"), "{output:?}");
    assert!(output.contains("☑"), "{output:?}");
    assert!(output.contains("□"), "{output:?}");
    assert!(
        output.find("active").unwrap() < output.find("11:32").unwrap(),
        "{output:?}"
    );
    assert!(
        output.find("done").unwrap() < output.find("12:10").unwrap(),
        "{output:?}"
    );
    assert!(
        output.find("pending").unwrap() < output.find("12:30").unwrap(),
        "{output:?}"
    );
    assert!(output.contains("  · first note"), "{output:?}");
    assert!(output.contains("  · done note"), "{output:?}");
    assert!(!output.contains("\n  · done note\n"), "{output:?}");
}

#[test]
fn render_document_shows_note_emoji_without_color() {
    let document =
        parser::parse_log("## 2026-03-24\n\n- [ ] 11:32 active\n  - first note\n").unwrap();

    let output = show_command::render_document(&document, false);

    assert!(output.contains("□ active  11:32"), "{output:?}");
    assert!(output.contains("  · first note"), "{output:?}");
}
