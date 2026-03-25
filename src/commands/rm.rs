use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

use crate::commands::show::print_log_if_changed;
use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::store;

pub fn run(interactive: bool, id: Option<String>) -> Result<()> {
    if !interactive && id.is_none() {
        return Err(anyhow!("rm requires -i"));
    }

    let path = crate::log::paths::default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    if let Some(id) = id {
        store::delete_by_transient_id(&path, &id)?;
    } else {
        let mut picker = TerminalPicker;
        run_terminal_at_path(&path, &mut picker)?;
    }
    print_log_if_changed(&path, &before)
}

pub trait Confirm {
    fn confirm(&mut self) -> Result<bool>;
}

pub fn run_terminal_at_path(path: &Path, picker: &mut TerminalPicker) -> Result<()> {
    let entries = store::collect_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no entries found"));
    }

    let Some(index) = picker.pick_for_delete(&entries)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    store::delete_entry(path, target)
}

pub fn run_interactive_at_path(
    path: &Path,
    picker: &mut impl Picker,
    confirmer: &mut impl Confirm,
) -> Result<()> {
    let entries = store::collect_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no entries found"));
    }

    let Some(index) = picker.pick(&entries)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    if confirmer.confirm()? {
        store::delete_entry(path, target)
    } else {
        Ok(())
    }
}
