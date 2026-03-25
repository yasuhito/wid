use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::commands::show::print_log_if_changed;
use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::{paths::default_log_path, store};

pub trait SummaryEditor {
    fn edit_summary(&mut self, initial: &str) -> Result<String>;
}

pub struct TerminalSummaryEditor;

impl SummaryEditor for TerminalSummaryEditor {
    fn edit_summary(&mut self, initial: &str) -> Result<String> {
        if io::stdin().is_terminal() {
            let mut editor = DefaultEditor::new().context("failed to initialize line editor")?;
            let summary = editor.readline_with_initial("summary: ", (initial, ""));
            return match summary {
                Ok(line) => validate_summary(line),
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    Err(anyhow!("empty summary"))
                }
                Err(error) => Err(error).context("failed to read summary"),
            };
        }

        eprint!("summary [{initial}]: ");
        io::stderr().flush().context("failed to flush prompt")?;

        let mut summary = String::new();
        io::stdin()
            .lock()
            .read_line(&mut summary)
            .context("failed to read summary")?;

        let summary = summary.trim_end_matches(['\r', '\n']);
        validate_summary(summary.to_string())
    }
}

pub fn run(interactive: bool) -> Result<()> {
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    let mut picker = TerminalPicker;
    let mut editor = TerminalSummaryEditor;
    run_at_path(&path, interactive, &mut picker, &mut editor)?;
    print_log_if_changed(&path, &before)
}

pub fn run_at_path(
    path: &Path,
    interactive: bool,
    picker: &mut impl Picker,
    editor: &mut impl SummaryEditor,
) -> Result<()> {
    if interactive {
        run_interactive_at_path(path, picker, editor)
    } else {
        run_default_at_path(path, editor)
    }
}

fn run_default_at_path(path: &Path, editor: &mut impl SummaryEditor) -> Result<()> {
    let entries = store::collect_entries(path)?;
    let target = entries
        .iter()
        .find(|entry| entry.state.is_active())
        .or_else(|| entries.last())
        .ok_or_else(|| anyhow!("no entry found"))?;

    let updated = validate_summary(editor.edit_summary(&target.summary)?)?;
    store::edit_entry_summary(path, target, &updated)
}

fn run_interactive_at_path(
    path: &Path,
    picker: &mut impl Picker,
    editor: &mut impl SummaryEditor,
) -> Result<()> {
    let entries = store::collect_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no entry found"));
    }

    let default_index = entries
        .iter()
        .position(|entry| entry.state.is_active())
        .unwrap_or(entries.len().saturating_sub(1));

    let Some(index) = picker.pick_with_selected(&entries, default_index)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    let updated = validate_summary(editor.edit_summary(&target.summary)?)?;
    store::edit_entry_summary(path, target, &updated)
}

fn validate_summary(summary: String) -> Result<String> {
    if summary.trim().is_empty() {
        return Err(anyhow!("empty summary"));
    }

    Ok(summary)
}
