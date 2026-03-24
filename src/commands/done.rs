use anyhow::Result;
use chrono::Local;

use crate::log::{paths::default_log_path, store::mark_last_unfinished_entry_done};

pub fn run() -> Result<()> {
    let now = Local::now();
    let timestamp = now.format("%F %R").to_string();
    let path = default_log_path()?;
    mark_last_unfinished_entry_done(&path, &timestamp)
}
