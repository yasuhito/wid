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

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use format::format_entry;
use model::{Entry, EntryState};
use parser::parse_log;
use paths::{default_log_path, default_log_path_from_home};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct HomeGuard {
    original: Option<OsString>,
}

impl HomeGuard {
    fn set(home: &PathBuf) -> Self {
        let original = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", home);
        }
        Self { original }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        unsafe {
            match self.original.take() {
                Some(value) => std::env::set_var("HOME", value),
                None => std::env::remove_var("HOME"),
            }
        }
    }
}

#[test]
fn format_entry_renders_bullet_line() {
    let entry = Entry {
        time: "11:32".into(),
        summary: "fix failing CI".into(),
        tags: Vec::new(),
        state: EntryState::Pending,
        notes: Vec::new(),
    };

    assert_eq!(format_entry(&entry), "- [ ] 11:32 fix failing CI");
}

#[test]
fn format_entry_renders_active_checkbox_line() {
    let entry = Entry {
        time: "11:32".into(),
        summary: "fix failing CI".into(),
        tags: Vec::new(),
        state: EntryState::Active,
        notes: Vec::new(),
    };

    assert_eq!(format_entry(&entry), "- [>] 11:32 fix failing CI");
}

#[test]
fn format_entry_renders_completed_checkbox_line() {
    let entry = Entry {
        time: "11:32".into(),
        summary: "fix failing CI".into(),
        tags: Vec::new(),
        state: EntryState::Done,
        notes: Vec::new(),
    };

    assert_eq!(format_entry(&entry), "- [x] 11:32 fix failing CI");
}

#[test]
fn default_log_path_uses_local_share_wid_log_md() {
    let home = PathBuf::from("/tmp/example-home");

    assert_eq!(
        default_log_path_from_home(&home),
        home.join(".local/share/wid/log.md")
    );
}

#[test]
fn default_log_path_reads_home_directory() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = PathBuf::from("/tmp/example-home");
    let _home_guard = HomeGuard::set(&home);
    let path = default_log_path().unwrap();

    assert_eq!(path, home.join(".local/share/wid/log.md"));
}

#[test]
fn parse_markdown_sections_and_entries() {
    let input = "## 2026-03-24\n\n- [ ] 11:32 fix failing CI @wid @agent\n  - follow-up note\n- [>] 11:50 address review feedback\n- [x] 12:10 rework implementation plan @md-edit\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].date, "2026-03-24");
    assert_eq!(doc.days[0].entries.len(), 3);
    assert_eq!(doc.days[0].entries[0].time, "11:32");
    assert_eq!(doc.days[0].entries[0].summary, "fix failing CI");
    assert_eq!(doc.days[0].entries[0].tags, vec!["wid", "agent"]);
    assert_eq!(doc.days[0].entries[0].state, EntryState::Pending);
    assert_eq!(doc.days[0].entries[0].notes, vec!["follow-up note"]);
    assert_eq!(doc.days[0].entries[1].summary, "address review feedback");
    assert!(doc.days[0].entries[1].tags.is_empty());
    assert_eq!(doc.days[0].entries[1].state, EntryState::Active);
    assert_eq!(doc.days[0].entries[2].summary, "rework implementation plan");
    assert_eq!(doc.days[0].entries[2].tags, vec!["md-edit"]);
    assert_eq!(doc.days[0].entries[2].state, EntryState::Done);
}

#[test]
fn format_entry_renders_tags_at_end_of_summary() {
    let entry = Entry {
        time: "11:32".into(),
        summary: "fix failing CI".into(),
        tags: vec!["wid".into(), "agent".into()],
        state: EntryState::Pending,
        notes: Vec::new(),
    };

    assert_eq!(
        format_entry(&entry),
        "- [ ] 11:32 fix failing CI @wid @agent"
    );
}

#[test]
fn parse_ignores_unrelated_lines() {
    let input = "random note\n## 2026-03-24\n\n- [ ] 09:00 start task\nnot markdown\n- [x] 09:30 in progress\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].entries.len(), 2);
    assert_eq!(doc.days[0].entries[0].summary, "start task");
    assert_eq!(doc.days[0].entries[1].summary, "in progress");
}

#[test]
fn parse_rejects_old_done_syntax() {
    let input = "## 2026-03-24\n\n- 09:00 old style @done(2026-03-24 09:10)\n";

    let doc = parse_log(input).unwrap();

    assert!(doc.days[0].entries.is_empty());
}

#[test]
fn parse_non_date_headings_end_the_current_day_section() {
    let input = "## 2026-03-24\n\n- [ ] 09:00 start task\n\n## Notes\n\n- [ ] 09:30 ignored item\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].entries.len(), 1);
    assert_eq!(doc.days[0].entries[0].summary, "start task");
}

#[test]
fn parse_accepts_day_headings_and_entries_with_trailing_spaces() {
    let input = "## 2026-03-24   \n\n- [ ] 09:00 start task   \n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].date, "2026-03-24");
    assert_eq!(doc.days[0].entries.len(), 1);
    assert_eq!(doc.days[0].entries[0].summary, "start task");
}

#[test]
fn parse_empty_input_returns_zero_days() {
    let doc = parse_log("").unwrap();

    assert!(doc.days.is_empty());
}

#[test]
fn parse_rejects_missing_space_after_time() {
    let input = "# wid log\n\n## 2026-03-24\n\n- [ ] 11:32summary\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert!(doc.days[0].entries.is_empty());
}
