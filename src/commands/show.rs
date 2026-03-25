use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use anyhow::Context;
use crossterm::style::Stylize;
use serde_json::json;

use crate::log::{
    model::{EntryState, LogDocument},
    store::{load_log, load_log_at_path},
};

pub fn run(json_output: bool) -> anyhow::Result<()> {
    let document = load_log()?;
    let output = if json_output {
        render_document_json(&document)
    } else {
        render_document(&document, io::stdout().is_terminal())
    };

    io::stdout()
        .write_all(output.as_bytes())
        .context("failed to write log output")?;

    Ok(())
}

pub fn render_document(document: &LogDocument, colorize: bool) -> String {
    let mut output = String::new();
    let non_empty_days: Vec<_> = document
        .days
        .iter()
        .filter(|day| !day.entries.is_empty())
        .collect();

    for (index, day) in non_empty_days.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }

        output.push_str(&format!("## {}\n\n", day.date));
        for entry in &day.entries {
            output.push_str(&render_entry_line(
                entry.state,
                &entry.time,
                &entry.summary,
                colorize,
            ));
            output.push('\n');
            for note in &entry.notes {
                output.push_str(&render_note_line(entry.state, note, colorize));
                output.push('\n');
            }
        }
    }

    output
}

pub fn render_document_json(document: &LogDocument) -> String {
    let days: Vec<_> = document
        .days
        .iter()
        .filter(|day| !day.entries.is_empty())
        .map(|day| {
            json!({
                "date": day.date,
                "entries": day.entries.iter().map(|entry| {
                    json!({
                        "id": entry.transient_id(&day.date),
                        "state": entry.state.as_str(),
                        "time": entry.time,
                        "summary": entry.summary,
                        "notes": entry.notes.iter().enumerate().map(|(index, note)| {
                            json!({
                                "id": entry.transient_note_id(&day.date, index, note),
                                "text": note,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>()
            })
        })
        .collect();

    serde_json::to_string_pretty(&json!({ "days": days })).expect("json output should serialize")
}

pub fn print_log_at_path(path: &Path) -> anyhow::Result<()> {
    let document = load_log_at_path(path)?;
    let output = render_document(&document, io::stdout().is_terminal());

    io::stdout()
        .write_all(output.as_bytes())
        .context("failed to write log output")?;

    Ok(())
}

pub fn print_log_if_changed(path: &Path, before: &str) -> anyhow::Result<()> {
    let after = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read log at {}", path.display()));
        }
    };

    if after != before {
        print_log_at_path(path)?;
    }

    Ok(())
}

fn render_entry_line(state: EntryState, time: &str, summary: &str, colorize: bool) -> String {
    let line = format!("- {} {} {}", state.checkbox(), time, summary);
    if !colorize {
        return line;
    }

    match state {
        EntryState::Pending => line,
        EntryState::Active => format!("{}", line.with(crossterm::style::Color::Yellow)),
        EntryState::Done => format!("{}", line.dark_grey()),
    }
}

fn render_note_line(state: EntryState, note: &str, colorize: bool) -> String {
    let line = format!("  - {note}");
    if !colorize {
        return line;
    }

    match state {
        EntryState::Pending => line,
        EntryState::Active => format!("{}", line.with(crossterm::style::Color::Yellow)),
        EntryState::Done => format!("{}", line.dark_grey()),
    }
}
