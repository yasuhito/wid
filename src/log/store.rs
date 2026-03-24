use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::format::{format_day_section, format_entry, format_log};
use super::model::{DaySection, Entry, EntryState, LogDocument, LogEntry, UnfinishedEntry};
use super::parser::{parse_day_heading, parse_entry_line, parse_log};
use super::paths::default_log_path;

pub fn load_log() -> Result<LogDocument> {
    let path = default_log_path()?;
    load_log_from_path(&path)
}

fn load_log_from_path(path: &Path) -> Result<LogDocument> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_log(&contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LogDocument::default()),
        Err(error) => Err(error).with_context(|| format!("failed to read log at {}", path.display())),
    }
}

pub fn save_log(document: &LogDocument) -> Result<()> {
    let path = default_log_path()?;
    save_log_to_path(&path, document)
}

pub fn collect_unfinished_entries(path: &Path) -> Result<Vec<UnfinishedEntry>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read log at {}", path.display()));
        }
    };

    Ok(collect_unfinished_entries_from_contents(&contents))
}

pub fn collect_entries(path: &Path) -> Result<Vec<LogEntry>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read log at {}", path.display()));
        }
    };

    Ok(collect_entries_from_contents(&contents))
}

pub fn append_log_entry(path: &Path, date: &str, time: &str, summary: &str) -> Result<()> {
    let entry = Entry {
        time: time.to_string(),
        summary: summary.to_string(),
        state: EntryState::Active,
    };
    append_log_entry_to_path(path, date, &entry)
}

pub fn mark_last_unfinished_entry_done(path: &Path, _timestamp: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    if let Some(target) = collect_active_entry_from_contents(&contents) {
        return mark_entry_done_with_contents(path, &contents, target.start, target.end);
    }

    let Some(target) = collect_unfinished_entries_from_contents(&contents).last().cloned() else {
        return Err(anyhow!("no unfinished entry found"));
    };

    mark_entry_done_with_contents(path, &contents, target.start, target.end)
}

pub fn mark_unfinished_entry_done(path: &Path, target: &UnfinishedEntry, _timestamp: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let fresh_target = collect_unfinished_entries_from_contents(&contents)
        .into_iter()
        .find(|entry| {
            entry.ordinal == target.ordinal
                && entry.date == target.date
                && entry.time == target.time
                && entry.summary == target.summary
        })
        .ok_or_else(|| anyhow!("selected entry changed before it could be completed"))?;

    mark_entry_done_with_contents(path, &contents, fresh_target.start, fresh_target.end)
}

pub fn delete_entry(path: &Path, target: &LogEntry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let fresh_target = collect_entries_from_contents(&contents)
        .into_iter()
        .find(|entry| {
            entry.ordinal == target.ordinal
                && entry.date == target.date
                && entry.time == target.time
                && entry.summary == target.summary
        })
        .ok_or_else(|| anyhow!("selected entry changed before it could be removed"))?;

    delete_entry_with_contents(path, &contents, &fresh_target)
}

pub fn focus_pending_entry(path: &Path, target: &UnfinishedEntry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let fresh_target = collect_unfinished_entries_from_contents(&contents)
        .into_iter()
        .find(|entry| {
            entry.ordinal == target.ordinal
                && entry.date == target.date
                && entry.time == target.time
                && entry.summary == target.summary
        })
        .ok_or_else(|| anyhow!("selected entry changed before it could be focused"))?;

    let demoted = demote_active_entries_in_contents(&contents);
    replace_entry_state_with_contents(path, &demoted, fresh_target.start, fresh_target.end, EntryState::Active)
}

fn save_log_to_path(path: &Path, document: &LogDocument) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = format_log(document);
    fs::write(path, contents)?;
    Ok(())
}

fn read_log_contents(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error).with_context(|| format!("failed to read log at {}", path.display())),
    }
}

fn collect_unfinished_entries_from_contents(contents: &str) -> Vec<UnfinishedEntry> {
    let mut entries = Vec::new();
    let mut line_start = 0;
    let mut current_date: Option<String> = None;

    for segment in contents.split_inclusive('\n') {
        let line_end = line_start + segment.len();
        let line = segment.trim_end();

        if let Some(date) = parse_day_heading(line) {
            current_date = Some(date.to_string());
        } else if line.starts_with("## ") {
            current_date = None;
        } else if let Some(date) = current_date.as_ref() {
            if let Some(entry) = parse_entry_line(line) {
                if entry.state.is_pending() {
                    entries.push(UnfinishedEntry {
                        date: date.clone(),
                        time: entry.time,
                        summary: entry.summary,
                        ordinal: entries.len(),
                        start: line_start,
                        end: line_end,
                    });
                }
            }
        }

        line_start = line_end;
    }

    entries
}

fn collect_entries_from_contents(contents: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let mut line_start = 0;
    let mut current_date: Option<String> = None;

    for segment in contents.split_inclusive('\n') {
        let line_end = line_start + segment.len();
        let line = segment.trim_end();

        if let Some(date) = parse_day_heading(line) {
            current_date = Some(date.to_string());
        } else if line.starts_with("## ") {
            current_date = None;
        } else if let Some(date) = current_date.as_ref() {
            if let Some(entry) = parse_entry_line(line) {
                entries.push(LogEntry {
                    date: date.clone(),
                    time: entry.time,
                    summary: entry.summary,
                    ordinal: entries.len(),
                    start: line_start,
                    end: line_end,
                });
            }
        }

        line_start = line_end;
    }

    entries
}

fn mark_entry_done_with_contents(path: &Path, contents: &str, start: usize, end: usize) -> Result<()> {
    replace_entry_state_with_contents(path, contents, start, end, EntryState::Done)
}

fn replace_entry_state_with_contents(
    path: &Path,
    contents: &str,
    start: usize,
    end: usize,
    state: EntryState,
) -> Result<()> {
    let mut updated = String::with_capacity(contents.len() + 4);
    updated.push_str(&contents[..start]);
    let segment = &contents[start..end];
    let body = segment.trim_end_matches(['\r', '\n']);
    let ending = &segment[body.len()..];
    let updated_body = replace_checkbox(body, state);
    updated.push_str(&updated_body);
    updated.push_str(ending);
    updated.push_str(&contents[end..]);

    fs::write(path, updated).with_context(|| format!("failed to write log at {}", path.display()))?;
    Ok(())
}

fn delete_entry_with_contents(path: &Path, contents: &str, target: &LogEntry) -> Result<()> {
    let mut updated = String::with_capacity(contents.len().saturating_sub(target.end - target.start));
    updated.push_str(&contents[..target.start]);
    updated.push_str(&contents[target.end..]);

    fs::write(path, updated).with_context(|| format!("failed to write log at {}", path.display()))?;
    Ok(())
}

fn append_log_entry_to_path(path: &Path, date: &str, entry: &Entry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    match fs::read_to_string(path) {
        Ok(contents) => {
            let contents = demote_active_entries_in_contents(&contents);
            append_to_existing_log(path, &contents, date, entry)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            write_new_log(path, date, entry)
        }
        Err(error) => Err(error).with_context(|| format!("failed to read log at {}", path.display())),
    }
}

fn write_new_log(path: &Path, date: &str, entry: &Entry) -> Result<()> {
    let document = LogDocument {
        days: vec![DaySection {
            date: date.to_string(),
            entries: vec![entry.clone()],
        }],
    };

    fs::write(path, format_log(&document))?;
    Ok(())
}

fn append_to_existing_log(path: &Path, contents: &str, date: &str, entry: &Entry) -> Result<()> {
    let document = parse_log(contents)?;
    let has_matching_day = document.days.iter().any(|day| day.date == date);
    if !has_matching_day {
        return append_new_day_at_eof(path, contents, date, entry);
    }

    if let Some(insertion_point) = find_existing_day_insertion_point(contents, date) {
        let line_ending = newline_ending(contents);
        let needs_separator = insertion_point > 0 && !contents[..insertion_point].ends_with(line_ending);
        let mut updated = String::with_capacity(contents.len() + entry.summary.len() + 16);
        updated.push_str(&contents[..insertion_point]);
        if needs_separator {
            updated.push_str(line_ending);
        }
        updated.push_str(&format!("{}{}", format_entry(entry), line_ending));
        updated.push_str(&contents[insertion_point..]);
        fs::write(path, updated)
            .with_context(|| format!("failed to write log at {}", path.display()))?;
        return Ok(());
    }

    append_new_day_at_eof(path, contents, date, entry)
}

fn append_new_day_at_eof(path: &Path, contents: &str, date: &str, entry: &Entry) -> Result<()> {
    let line_ending = newline_ending(contents);
    let mut updated = String::with_capacity(contents.len() + date.len() + entry.summary.len() + 32);
    updated.push_str(contents);
    if !updated.is_empty() && !updated.ends_with(line_ending) {
        updated.push_str(line_ending);
    }
    if updated.ends_with(line_ending) && !updated.ends_with(&format!("{line_ending}{line_ending}")) {
        updated.push_str(line_ending);
    }
    updated.push_str(&format_day_section(&DaySection {
        date: date.to_string(),
        entries: vec![entry.clone()],
    }).replace('\n', line_ending));

    fs::write(path, updated)
        .with_context(|| format!("failed to write log at {}", path.display()))?;
    Ok(())
}

fn find_existing_day_insertion_point(contents: &str, date: &str) -> Option<usize> {
    let target_heading = format!("## {date}");
    let mut line_start = 0;
    let mut heading_end = None;
    let mut last_non_empty_line_end = None;
    let mut seen_target_heading = false;

    for line in contents.split_inclusive('\n') {
        let line_text = line.strip_suffix('\n').unwrap_or(line).trim_end();

        if !seen_target_heading {
            if line_text == target_heading {
                seen_target_heading = true;
                heading_end = Some(line_start + line.len());
            }
        } else if line_text.starts_with("## ") {
            break;
        } else {
            if !line_text.is_empty() {
                last_non_empty_line_end = Some(line_start + line.len());
            }
        }

        line_start += line.len();
    }

    if !seen_target_heading {
        return None;
    }

    last_non_empty_line_end.or(heading_end)
}

fn newline_ending(contents: &str) -> &str {
    if contents.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn collect_active_entry_from_contents(contents: &str) -> Option<LogEntry> {
    collect_entries_from_contents(contents)
        .into_iter()
        .find(|entry| entry_line_state(contents, entry.start, entry.end) == Some(EntryState::Active))
}

fn demote_active_entries_in_contents(contents: &str) -> String {
    let mut updated = String::with_capacity(contents.len());
    for segment in contents.split_inclusive('\n') {
        let body = segment.trim_end_matches(['\r', '\n']);
        let ending = &segment[body.len()..];
        if body.starts_with("- [>] ") {
            updated.push_str(&body.replacen("[>]", "[ ]", 1));
        } else {
            updated.push_str(body);
        }
        updated.push_str(ending);
    }

    updated
}

fn entry_line_state(contents: &str, start: usize, end: usize) -> Option<EntryState> {
    let line = contents[start..end].trim_end();
    parse_entry_line(line).map(|entry| entry.state)
}

fn replace_checkbox(line: &str, state: EntryState) -> String {
    if line.contains("[>]") {
        line.replacen("[>]", state.checkbox(), 1)
    } else if line.contains("[x]") {
        line.replacen("[x]", state.checkbox(), 1)
    } else {
        line.replacen("[ ]", state.checkbox(), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        dir.push(format!("wid-{name}-{stamp}-{}", std::process::id()));
        dir
    }

    #[test]
    fn load_log_treats_missing_file_as_empty() {
        let dir = unique_temp_dir("missing");
        let path = dir.join("log.md");

        let loaded = load_log_from_path(&path).unwrap();

        assert_eq!(loaded, LogDocument::default());
    }

    #[test]
    fn load_log_returns_context_for_other_read_errors() {
        let dir = unique_temp_dir("read-error");
        fs::create_dir_all(&dir).unwrap();

        let error = load_log_from_path(&dir).unwrap_err();
        let message = format!("{error:#}");

        assert!(message.contains("failed to read log at"));
    }

    #[test]
    fn save_log_creates_parent_directory() {
        let dir = unique_temp_dir("save");
        let path = dir.join(".local/share/wid/log.md");
        let document = LogDocument::default();

        save_log_to_path(&path, &document).unwrap();

        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), format_log(&document));
    }
}
