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
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n",
    )
    .unwrap();

    let output = run_wid(&home, &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "# wid log\n\n## 2026-03-24\n\n- 11:32 CI が落ちていたので修正\n- 12:10 実装方針を見直した\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}
