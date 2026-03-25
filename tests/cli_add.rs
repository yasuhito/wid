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

use std::fs;
use std::io::Write;
use std::path::PathBuf;
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

    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();

    if let Some(input) = stdin {
        child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    }

    child.wait_with_output().unwrap()
}

fn log_path(home: &PathBuf) -> PathBuf {
    home.join(".local/share/wid/log.md")
}

#[test]
fn add_command_appends_pending_entry() {
    let home = unique_temp_dir("add-join");
    let output = run_wid(&home, &["add", "あとで", "確認する"], None);

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("## "), "{stdout}");
    assert!(stdout.contains("- [ ] "), "{stdout}");
    assert!(stdout.contains("あとで 確認する"), "{stdout}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(contents.contains("- [ ] "), "{contents}");
    assert!(contents.contains("あとで 確認する"), "{contents}");
}

#[test]
fn add_command_reads_single_line_when_no_args_are_given() {
    let home = unique_temp_dir("add-stdin");
    let output = run_wid(&home, &["add"], Some("あとで整理する\n"));

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(log_path(&home)).unwrap();
    assert!(contents.contains("- [ ] "), "{contents}");
    assert!(contents.contains("あとで整理する"), "{contents}");
}

#[test]
fn add_command_keeps_existing_active_entry() {
    let home = unique_temp_dir("add-keep-active");
    let path = log_path(&home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# wid log\n\n## 2026-03-25\n\n- [>] 09:00 active task\n",
    )
    .unwrap();

    let output = run_wid(&home, &["add", "backlog", "item"], None);

    assert!(output.status.success(), "{output:?}");
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("- [>] 09:00 active task"), "{contents}");
    assert!(contents.contains("- [ ] "), "{contents}");
    assert!(contents.contains("backlog item"), "{contents}");
}
