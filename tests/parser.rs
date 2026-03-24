#[allow(dead_code)]
#[path = "../src/log/model.rs"]
mod model;
#[allow(dead_code)]
#[path = "../src/log/format.rs"]
mod format;
#[path = "../src/log/paths.rs"]
mod paths;
#[path = "../src/log/parser.rs"]
mod parser;

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use format::format_entry;
use model::Entry;
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
        summary: "CI が落ちていたので修正".into(),
    };

    assert_eq!(format_entry(&entry), "- 11:32 CI が落ちていたので修正");
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
    let input = "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].date, "2026-03-24");
    assert_eq!(doc.days[0].entries.len(), 2);
    assert_eq!(doc.days[0].entries[0].time, "11:32");
    assert_eq!(doc.days[0].entries[0].summary, "CI が落ちていたので修正");
    assert_eq!(doc.days[0].entries[1].summary, "実装方針を見直した");
}

#[test]
fn parse_ignores_unrelated_lines() {
    let input = "# wid log\n\nrandom note\n## 2026-03-24\n\n- 09:00 着手\nnot markdown\n- 09:30 進行\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].entries.len(), 2);
    assert_eq!(doc.days[0].entries[0].summary, "着手");
    assert_eq!(doc.days[0].entries[1].summary, "進行");
}

#[test]
fn parse_non_date_headings_end_the_current_day_section() {
    let input =
        "# wid log\n\n## 2026-03-24\n\n- 09:00 着手\n\n## Notes\n\n- 09:30 無視される\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].entries.len(), 1);
    assert_eq!(doc.days[0].entries[0].summary, "着手");
}

#[test]
fn parse_accepts_day_headings_and_entries_with_trailing_spaces() {
    let input = "# wid log\n\n## 2026-03-24   \n\n- 09:00 着手   \n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert_eq!(doc.days[0].date, "2026-03-24");
    assert_eq!(doc.days[0].entries.len(), 1);
    assert_eq!(doc.days[0].entries[0].summary, "着手");
}

#[test]
fn parse_empty_or_header_only_log_returns_zero_days() {
    let doc = parse_log("# wid log\n").unwrap();

    assert!(doc.days.is_empty());
}

#[test]
fn parse_empty_input_returns_zero_days() {
    let doc = parse_log("").unwrap();

    assert!(doc.days.is_empty());
}

#[test]
fn parse_rejects_missing_space_after_time() {
    let input = "# wid log\n\n## 2026-03-24\n\n- 11:32summary\n";

    let doc = parse_log(input).unwrap();

    assert_eq!(doc.days.len(), 1);
    assert!(doc.days[0].entries.is_empty());
}
