use anyhow::Result;

use crate::commands::now::resolve_summary;
use crate::log::{paths::default_log_path, store::append_note_to_latest_open_entry};

pub fn run(text: Vec<String>) -> Result<()> {
    let path = default_log_path()?;
    run_at_path(&path, text)
}

pub fn run_at_path(path: &std::path::Path, text: Vec<String>) -> Result<()> {
    let note = resolve_summary(text)?;
    append_note_to_latest_open_entry(path, &note)
}
