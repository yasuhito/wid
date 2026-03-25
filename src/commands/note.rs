use std::fs;

use anyhow::Result;

use crate::commands::now::resolve_summary;
use crate::commands::show::print_log_if_changed;
use crate::log::{paths::default_log_path, store::append_note_to_latest_open_entry};

pub fn run(text: Vec<String>) -> Result<()> {
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    run_at_path(&path, text)?;
    print_log_if_changed(&path, &before)
}

pub fn run_at_path(path: &std::path::Path, text: Vec<String>) -> Result<()> {
    let note = resolve_summary(text)?;
    append_note_to_latest_open_entry(path, &note)
}
