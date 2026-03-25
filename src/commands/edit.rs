use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::commands::show::{print_log_if_changed, render_entry_summary};
use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::{
    model::{Entry, RemovableKind},
    paths::default_log_path,
    store,
};

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

pub fn run(interactive: bool, id: Option<String>, text: Vec<String>) -> Result<()> {
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    let mut picker = TerminalPicker;
    let mut editor = TerminalSummaryEditor;
    run_at_path_with_options(
        &path,
        interactive,
        id.as_deref(),
        text,
        &mut picker,
        &mut editor,
    )?;
    print_log_if_changed(&path, &before)
}

pub fn run_at_path(
    path: &Path,
    interactive: bool,
    picker: &mut impl Picker,
    editor: &mut impl SummaryEditor,
) -> Result<()> {
    run_at_path_with_options(path, interactive, None, Vec::new(), picker, editor)
}

pub fn run_at_path_with_options(
    path: &Path,
    interactive: bool,
    id: Option<&str>,
    text: Vec<String>,
    picker: &mut impl Picker,
    editor: &mut impl SummaryEditor,
) -> Result<()> {
    if let Some(id) = id {
        run_by_id_at_path_with_text(path, id, text, editor)
    } else if interactive {
        run_interactive_at_path(path, picker, editor)
    } else {
        run_default_at_path(path, editor)
    }
}

pub fn run_by_id_at_path(path: &Path, id: &str, editor: &mut impl SummaryEditor) -> Result<()> {
    run_by_id_at_path_with_text(path, id, Vec::new(), editor)
}

pub fn run_by_id_at_path_with_text(
    path: &Path,
    id: &str,
    text: Vec<String>,
    editor: &mut impl SummaryEditor,
) -> Result<()> {
    let targets = store::collect_removable_targets(path)?;
    let target = targets
        .iter()
        .find(|target| match target.kind {
            RemovableKind::Entry => {
                let entry = Entry {
                    time: target.time.clone(),
                    summary: target.summary.clone(),
                    tags: target.tags.clone(),
                    state: target.state,
                    notes: Vec::new(),
                };
                entry.transient_id(&target.date) == id
            }
            RemovableKind::Note => {
                let entry = Entry {
                    time: target.time.clone(),
                    summary: target.summary.clone(),
                    tags: target.tags.clone(),
                    state: target.state,
                    notes: Vec::new(),
                };
                target
                    .note_text
                    .as_deref()
                    .map(|note| entry.transient_note_id(&target.date, note) == id)
                    .unwrap_or(false)
            }
        })
        .ok_or_else(|| anyhow!("item changed or not found"))?;

    let updated = if text.is_empty() {
        let initial = match target.kind {
            RemovableKind::Entry => render_entry_summary(&target.summary, &target.tags),
            RemovableKind::Note => target.note_text.as_deref().unwrap_or_default().to_string(),
        };
        validate_summary(editor.edit_summary(&initial)?)?
    } else {
        validate_summary(text.join(" "))?
    };

    store::edit_by_transient_id(path, id, &updated)
}

fn run_default_at_path(path: &Path, editor: &mut impl SummaryEditor) -> Result<()> {
    let entries = store::collect_entries(path)?;
    let target = entries
        .iter()
        .find(|entry| entry.state.is_active())
        .or_else(|| entries.last())
        .ok_or_else(|| anyhow!("no entry found"))?;

    let initial = render_entry_summary(&target.summary, &target.tags);
    let updated = validate_summary(editor.edit_summary(&initial)?)?;
    store::edit_entry_summary(path, target, &updated)
}

fn run_interactive_at_path(
    path: &Path,
    picker: &mut impl Picker,
    editor: &mut impl SummaryEditor,
) -> Result<()> {
    let targets = store::collect_removable_targets(path)?;
    if targets.is_empty() {
        return Err(anyhow!("no entry found"));
    }

    let default_index = targets
        .iter()
        .position(|target| target.kind == RemovableKind::Entry && target.state.is_active())
        .unwrap_or(targets.len().saturating_sub(1));

    let Some(index) = picker.pick_with_selected(&targets, default_index)? else {
        return Ok(());
    };

    let Some(target) = targets.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    let initial = match target.kind {
        RemovableKind::Entry => {
            return store::edit_removable_target_text(
                path,
                target,
                &validate_summary(
                    editor.edit_summary(&render_entry_summary(&target.summary, &target.tags))?,
                )?,
            );
        }
        RemovableKind::Note => target.note_text.as_deref().unwrap_or_default(),
    };
    let updated = validate_summary(editor.edit_summary(initial)?)?;
    store::edit_removable_target_text(path, target, &updated)
}

fn validate_summary(summary: String) -> Result<String> {
    if summary.trim().is_empty() {
        return Err(anyhow!("empty summary"));
    }

    Ok(summary)
}
