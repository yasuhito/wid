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
#[path = "../src/commands/now.rs"]
mod now_command;
#[path = "../src/commands/note.rs"]
mod note_command;
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
}

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
fn note_appends_to_active_entry() {
    let dir = unique_temp_dir("note-active");
    let path = dir.join("log.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "## 2026-03-25\n\n- [>] 08:01 active item\n- [ ] 08:12 pending item\n",
    )
    .unwrap();

    note_command::run_at_path(&path, vec!["first".into(), "note".into()]).unwrap();

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

    note_command::run_at_path(&path, vec!["remember".into(), "this".into()]).unwrap();

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

    note_command::run_at_path(&path, vec!["second".into(), "note".into()]).unwrap();

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

    let error = note_command::run_at_path(&path, vec!["orphan".into()]).unwrap_err();

    assert!(format!("{error:#}").contains("note target"), "{error:#}");
}
