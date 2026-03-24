use anyhow::{anyhow, Result};
use std::path::Path;

use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::{paths::default_log_path, store};

pub fn run(interactive: bool) -> Result<()> {
    if !interactive {
        return Err(anyhow!("focus requires -i"));
    }

    let path = default_log_path()?;
    let mut picker = TerminalPicker;
    run_interactive_at_path(&path, &mut picker)
}

pub fn run_interactive_at_path(path: &Path, picker: &mut impl Picker) -> Result<()> {
    let entries = store::collect_unfinished_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no pending entry found"));
    }

    let Some(index) = picker.pick(&entries)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    store::focus_pending_entry(path, target)
}
