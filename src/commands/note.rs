use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use anyhow::{Result, anyhow};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::commands::now::resolve_summary;
use crate::commands::show::print_log_if_changed;
use crate::interactive::done_picker::{GroupedLogEntryPicker, TerminalPicker};
use crate::log::{
    paths::default_log_path,
    store::{append_note_by_transient_id, append_note_to_latest_open_entry, collect_entries},
};

pub trait NoteEditor {
    fn edit_note(&mut self) -> Result<String>;
}

pub struct TerminalNoteEditor;

impl NoteEditor for TerminalNoteEditor {
    fn edit_note(&mut self) -> Result<String> {
        if io::stdin().is_terminal() {
            let mut editor = DefaultEditor::new()?;
            let note = editor.readline("note: ");
            return match note {
                Ok(line) => validate_note(line),
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    Err(anyhow!("empty note"))
                }
                Err(error) => Err(error.into()),
            };
        }

        eprint!("note: ");
        io::stderr().flush()?;
        let mut note = String::new();
        io::stdin().lock().read_line(&mut note)?;
        validate_note(note.trim_end_matches(['\r', '\n']).to_string())
    }
}

pub fn run(text: Vec<String>, id: Option<String>, interactive: bool) -> Result<()> {
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    let mut picker = TerminalPicker;
    let mut editor = TerminalNoteEditor;
    run_at_path_with_options(
        &path,
        text,
        id.as_deref(),
        interactive,
        &mut picker,
        &mut editor,
    )?;
    print_log_if_changed(&path, &before)
}

pub fn run_at_path(path: &Path, text: Vec<String>, id: Option<&str>) -> Result<()> {
    run_at_path_with_options(
        path,
        text,
        id,
        false,
        &mut TerminalPicker,
        &mut TerminalNoteEditor,
    )
}

pub fn run_at_path_with_options(
    path: &Path,
    text: Vec<String>,
    id: Option<&str>,
    interactive: bool,
    picker: &mut impl GroupedLogEntryPicker,
    editor: &mut impl NoteEditor,
) -> Result<()> {
    if interactive {
        return run_interactive_at_path(path, picker, editor);
    }

    let note = resolve_summary(text)?;
    if let Some(id) = id {
        append_note_by_transient_id(path, id, &note)
    } else {
        append_note_to_latest_open_entry(path, &note)
    }
}

pub fn run_interactive_at_path(
    path: &Path,
    picker: &mut impl GroupedLogEntryPicker,
    editor: &mut impl NoteEditor,
) -> Result<()> {
    let entries = collect_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no entry found"));
    }

    let Some(index) = picker.pick_grouped_entries(&entries, 0)? else {
        return Ok(());
    };

    let Some(target) = entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    let note = validate_note(editor.edit_note()?)?;
    append_note_by_transient_id(path, &target.transient_id(), &note)
}

fn validate_note(note: String) -> Result<String> {
    if note.trim().is_empty() {
        return Err(anyhow!("empty note"));
    }

    Ok(note)
}
