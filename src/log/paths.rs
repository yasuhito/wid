use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub fn default_log_path_from_home(home: &Path) -> PathBuf {
    home.join(".local/share/wid/log.md")
}

pub fn default_log_path() -> Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    Ok(default_log_path_from_home(Path::new(&home)))
}
