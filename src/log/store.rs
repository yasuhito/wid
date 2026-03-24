use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::format::{format_day_section, format_log};
use super::model::{DaySection, Entry, LogDocument};
use super::parser::parse_log;
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

pub fn append_log_entry(path: &Path, date: &str, time: &str, summary: &str) -> Result<()> {
    let entry = Entry {
        time: time.to_string(),
        summary: summary.to_string(),
    };
    append_log_entry_to_path(path, date, &entry)
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

fn append_log_entry_to_path(path: &Path, date: &str, entry: &Entry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    match fs::read_to_string(path) {
        Ok(contents) => append_to_existing_log(path, &contents, date, entry),
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
        updated.push_str(&format!("- {} {}{}", entry.time, entry.summary, line_ending));
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
