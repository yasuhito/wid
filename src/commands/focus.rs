use anyhow::{anyhow, Result};
use std::path::Path;

use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::{paths::default_log_path, store};

pub fn run(interactive: bool) -> Result<()> {
    let path = default_log_path()?;
    run_at_path(&path, interactive)
}

pub fn run_at_path(path: &Path, interactive: bool) -> Result<()> {
    if !interactive {
        return store::focus_latest_entry(path);
    }

    let mut picker = TerminalPicker;
    run_interactive_at_path(path, &mut picker)
}

pub fn run_interactive_at_path(path: &Path, picker: &mut impl Picker) -> Result<()> {
    let entries = store::collect_focus_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no pending entry found"));
    }

    let default_index = entries
        .iter()
        .position(|entry| entry.state.is_active())
        .unwrap_or(0);

    let Some(index) = picker.pick_with_selected(&entries, default_index)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    store::focus_entry(path, target)
}
