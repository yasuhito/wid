use std::io::{self, BufRead, Write};

use anyhow::{anyhow, Context, Result};
use chrono::Local;

use crate::log::{paths::default_log_path, store::append_log_entry};

pub fn run(text: Vec<String>) -> Result<()> {
    let summary = if text.is_empty() {
        read_summary_from_stdin()?
    } else {
        validate_summary(text.join(" "))?
    };

    let (date, time) = current_local_date_time()?;
    let path = default_log_path()?;
    append_log_entry(&path, &date, &time, &summary)
}

fn read_summary_from_stdin() -> Result<String> {
    eprint!("summary: ");
    io::stderr().flush().context("failed to flush prompt")?;

    let mut summary = String::new();
    io::stdin()
        .lock()
        .read_line(&mut summary)
        .context("failed to read summary")?;

    let summary = summary.trim_end_matches(['\r', '\n']);
    validate_summary(summary.to_string())
}

fn validate_summary(summary: String) -> Result<String> {
    if summary.trim().is_empty() {
        return Err(anyhow!("empty summary"));
    }

    Ok(summary)
}

fn current_local_date_time() -> Result<(String, String)> {
    let now = Local::now();
    let formatted = now.format("%F %R").to_string();
    let (date, time) = formatted
        .split_once(' ')
        .ok_or_else(|| anyhow!("failed to determine current local date and time"))?;

    if date.is_empty() || time.is_empty() {
        return Err(anyhow!("failed to determine current local date and time"));
    }

    Ok((date.to_string(), time.to_string()))
}
