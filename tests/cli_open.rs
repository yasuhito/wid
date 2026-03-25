#![allow(dead_code, unused_imports)]

#[path = "../src/commands/open.rs"]
mod open_command;
#[path = "../src/log/paths.rs"]
mod paths;
mod log {
    pub mod paths {
        pub use crate::paths::*;
    }
}

use std::fs;
use std::path::{Path, PathBuf};
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
fn open_targets_log_md_by_default() {
    let dir = unique_temp_dir("open-log");
    let home = dir.clone();
    let mut launcher = FakeLauncher::default();

    open_command::run_at_home(&home, false, Some("fake-editor"), &mut launcher).unwrap();

    assert_eq!(
        launcher.calls,
        vec![(
            "fake-editor".to_string(),
            home.join(".local/share/wid/log.md")
        )]
    );
    assert!(home.join(".local/share/wid/log.md").exists());
}

#[test]
fn open_targets_archive_md_with_flag() {
    let dir = unique_temp_dir("open-archive");
    let home = dir.clone();
    let mut launcher = FakeLauncher::default();

    open_command::run_at_home(&home, true, Some("fake-editor"), &mut launcher).unwrap();

    assert_eq!(
        launcher.calls,
        vec![(
            "fake-editor".to_string(),
            home.join(".local/share/wid/archive.md")
        )]
    );
    assert!(home.join(".local/share/wid/archive.md").exists());
}

#[test]
fn open_errors_when_editor_is_not_set() {
    let dir = unique_temp_dir("open-no-editor");
    let mut launcher = FakeLauncher::default();

    let error = open_command::run_at_home(&dir, false, None, &mut launcher).unwrap_err();

    assert!(format!("{error:#}").contains("EDITOR"), "{error:#}");
    assert!(launcher.calls.is_empty());
}

#[derive(Default)]
struct FakeLauncher {
    calls: Vec<(String, PathBuf)>,
}

impl open_command::EditorLauncher for FakeLauncher {
    fn launch(&mut self, editor: &str, path: &Path) -> anyhow::Result<()> {
        self.calls.push((editor.to_string(), path.to_path_buf()));
        Ok(())
    }
}
