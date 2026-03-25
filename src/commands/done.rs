use anyhow::{anyhow, Result};
use chrono::Local;
use std::fs;
use std::path::Path;

use crate::commands::show::print_log_if_changed;
use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::{paths::default_log_path, store};

pub fn run(interactive: bool) -> Result<()> {
    let now = Local::now();
    let timestamp = now.format("%F %R").to_string();
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();

    if interactive {
        let mut picker = TerminalPicker;
        run_interactive_at_path(&path, &timestamp, &mut picker)?;
    } else {
        store::mark_last_unfinished_entry_done(&path, &timestamp)?;
    }

    print_log_if_changed(&path, &before)
}

pub fn run_interactive_at_path(
    path: &Path,
    timestamp: &str,
    picker: &mut impl Picker,
) -> Result<()> {
    let entries = store::collect_focus_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no unfinished entry found"));
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
    store::mark_focus_entry_done(path, target, timestamp)
}
