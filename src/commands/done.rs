use anyhow::{anyhow, Result};
use chrono::Local;
use std::path::Path;

use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::{paths::default_log_path, store};

pub fn run(interactive: bool) -> Result<()> {
    let now = Local::now();
    let timestamp = now.format("%F %R").to_string();
    let path = default_log_path()?;

    if interactive {
        let mut picker = TerminalPicker;
        run_interactive_at_path(&path, &timestamp, &mut picker)
    } else {
        store::mark_last_unfinished_entry_done(&path, &timestamp)
    }
}

pub fn run_interactive_at_path(
    path: &Path,
    timestamp: &str,
    picker: &mut impl Picker,
) -> Result<()> {
    let entries = store::collect_unfinished_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no unfinished entry found"));
    }

    let Some(index) = picker.pick(&entries)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };
    store::mark_unfinished_entry_done(path, target, timestamp)
}
