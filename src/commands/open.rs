use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};

use crate::log::paths::{default_archive_path_from_home, default_log_path_from_home};

pub trait EditorLauncher {
    fn launch(&mut self, editor: &str, path: &Path) -> Result<()>;
}

pub struct SystemEditorLauncher;

impl EditorLauncher for SystemEditorLauncher {
    fn launch(&mut self, editor: &str, path: &Path) -> Result<()> {
        let status = Command::new(editor)
            .arg(path)
            .status()
            .with_context(|| format!("failed to launch editor `{editor}`"))?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("editor exited with status {status}"))
        }
    }
}

pub fn run(open_archive: bool) -> Result<()> {
    let home = env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    let mut launcher = SystemEditorLauncher;
    run_at_home(
        Path::new(&home),
        open_archive,
        env::var("EDITOR").ok().as_deref(),
        &mut launcher,
    )
}

pub fn run_at_home(
    home: &Path,
    open_archive: bool,
    editor: Option<&str>,
    launcher: &mut impl EditorLauncher,
) -> Result<()> {
    let Some(editor) = editor else {
        return Err(anyhow!("EDITOR is not set"));
    };

    let path: PathBuf = if open_archive {
        default_archive_path_from_home(home)
    } else {
        default_log_path_from_home(home)
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory at {}", parent.display()))?;
    }

    if !path.exists() {
        fs::write(&path, "")
            .with_context(|| format!("failed to create file at {}", path.display()))?;
    }

    launcher.launch(editor, &path)
}
