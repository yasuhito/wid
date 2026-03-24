use std::io::{self, IsTerminal, Write};

use anyhow::Context;
use crossterm::style::Stylize;

use crate::log::{
    format::format_log_header,
    model::{EntryState, LogDocument},
    store::load_log,
};

pub fn run() -> anyhow::Result<()> {
    let document = load_log()?;
    let output = render_document(&document, io::stdout().is_terminal());

    io::stdout()
        .write_all(output.as_bytes())
        .context("failed to write log output")?;

    Ok(())
}

pub fn render_document(document: &LogDocument, colorize: bool) -> String {
    let mut output = String::from(format_log_header());

    for (index, day) in document.days.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }

        output.push_str(&format!("## {}\n\n", day.date));
        for entry in &day.entries {
            output.push_str(&render_entry_line(entry.state, &entry.time, &entry.summary, colorize));
            output.push('\n');
        }
    }

    output
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
