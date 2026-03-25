use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::commands::now::resolve_summary;
use crate::commands::show::print_log_if_changed;
use crate::log::{
    paths::default_log_path,
    store::{append_note_by_transient_id, append_note_to_latest_open_entry},
};

pub fn run(text: Vec<String>, id: Option<String>) -> Result<()> {
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    run_at_path(&path, text, id.as_deref())?;
    print_log_if_changed(&path, &before)
}

pub fn run_at_path(path: &Path, text: Vec<String>, id: Option<&str>) -> Result<()> {
    let note = resolve_summary(text)?;
    if let Some(id) = id {
        append_note_by_transient_id(path, id, &note)
    } else {
        append_note_to_latest_open_entry(path, &note)
    }
}
