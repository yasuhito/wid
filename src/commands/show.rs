use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use anyhow::Context;
use chrono::{Local, NaiveDate};
use crossterm::style::Stylize;
use serde_json::json;

use crate::log::{
    model::{EntryState, LogDocument, format_note_display, format_summary_with_tags},
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

        let heading = render_day_heading(&day.date);
        output.push_str(&heading);
        output.push('\n');
        output.push_str(&render_day_separator(&heading));
        output.push('\n');
        for entry in &day.entries {
            let summary = render_entry_summary(&entry.summary, &entry.tags);
            output.push_str(&render_entry_line(
                entry.state,
                &entry.time,
                &summary,
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

fn render_day_heading(date: &str) -> String {
    let Ok(day) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return date.to_string();
    };

    let label = day.format("%Y-%m-%d %a").to_string();
    let today = Local::now().date_naive();

    if day == today {
        format!("Today · {label}")
    } else if day == today.pred_opt().unwrap_or(today) {
        format!("Yesterday · {label}")
    } else {
        label
    }
}

fn render_day_separator(heading: &str) -> String {
    "─".repeat(heading.chars().count())
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
                        "tags": entry.tags,
                        "notes": entry.notes.iter().map(|note| {
                            json!({
                                "id": entry.transient_note_id(&day.date, note),
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
    let marker = state.display_marker();
    let body = render_entry_body(summary, time);
    if !colorize {
        return format!("{marker} {body}");
    }

    match state {
        EntryState::Pending => format!("{marker} {summary}  {}", time.dark_grey()),
        EntryState::Active => format!(
            "{} {}  {}",
            marker.with(crossterm::style::Color::Yellow),
            summary.with(crossterm::style::Color::Yellow),
            time.with(crossterm::style::Color::DarkYellow)
        ),
        EntryState::Done => format!("{} {}  {}", marker.dark_grey(), summary.dark_grey(), time.dark_grey()),
    }
}

fn render_entry_body(summary: &str, time: &str) -> String {
    format!("{summary}  {time}")
}

pub fn render_entry_summary(summary: &str, tags: &[String]) -> String {
    format_summary_with_tags(summary, tags)
}

fn render_note_line(state: EntryState, note: &str, colorize: bool) -> String {
    let line = format_note_display(note);
    if !colorize {
        return line;
    }

    match state {
        EntryState::Pending => line,
        EntryState::Active => format!("{}", line.with(crossterm::style::Color::Yellow)),
        EntryState::Done => format!("{}", line.dark_grey()),
    }
}
