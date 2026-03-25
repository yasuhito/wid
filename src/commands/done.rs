use anyhow::{Result, anyhow};
use chrono::Local;
use std::fs;
use std::path::Path;

use crate::commands::show::print_log_if_changed;
use crate::interactive::done_picker::{DoneStatePicker, TerminalPicker};
use crate::log::{paths::default_log_path, store};

pub fn run(interactive: bool, id: Option<String>) -> Result<()> {
    let now = Local::now();
    let timestamp = now.format("%F %R").to_string();
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();

    if let Some(id) = id {
        store::mark_entry_done_by_transient_id(&path, &id)?;
    } else if interactive {
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
    picker: &mut impl DoneStatePicker,
) -> Result<()> {
    let entries = store::collect_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no entry found"));
    }

    let default_index = entries
        .iter()
        .position(|entry| entry.state.is_active())
        .unwrap_or(0);

    let Some(states) = picker.pick_done_states(&entries, default_index)? else {
        return Ok(());
    };

    if states.len() != entries.len() {
        return Err(anyhow!("invalid state selection"));
    };

    let updates: Vec<_> = entries
        .iter()
        .zip(states)
        .filter_map(|(entry, state)| {
            if entry.state == state {
                None
            } else {
                Some((entry.clone(), state))
            }
        })
        .collect();

    if updates.is_empty() {
        return Ok(());
    }

    let _ = timestamp;
    store::apply_entry_state_updates(path, &updates)
}
