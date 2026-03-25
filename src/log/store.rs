use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};

use super::format::{format_day_section, format_entry, format_log};
use super::model::{
    DaySection, Entry, EntryState, FocusEntry, LogDocument, LogEntry, RemovableKind,
    RemovableTarget, UnfinishedEntry,
};
use super::parser::{parse_day_heading, parse_entry_line, parse_log, parse_note_line};
use super::paths::{default_archive_path, default_log_path};

pub fn load_log() -> Result<LogDocument> {
    let path = default_log_path()?;
    load_log_at_path(&path)
}

pub fn load_log_at_path(path: &Path) -> Result<LogDocument> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_log(&contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LogDocument::default()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to read log at {}", path.display()))
        }
    }
}

pub fn save_log(document: &LogDocument) -> Result<()> {
    let path = default_log_path()?;
    save_log_to_path(&path, document)
}

pub fn archive_done_entries() -> Result<()> {
    let log_path = default_log_path()?;
    let archive_path = default_archive_path()?;
    archive_done_entries_at_paths(&log_path, &archive_path)
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

pub fn collect_removable_targets(path: &Path) -> Result<Vec<RemovableTarget>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read log at {}", path.display()));
        }
    };

    Ok(collect_removable_targets_from_contents(&contents))
}

pub fn collect_focus_entries(path: &Path) -> Result<Vec<FocusEntry>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read log at {}", path.display()));
        }
    };

    Ok(collect_focus_entries_from_contents(&contents))
}

pub fn append_log_entry(path: &Path, date: &str, time: &str, summary: &str) -> Result<()> {
    let entry = Entry {
        time: time.to_string(),
        summary: summary.to_string(),
        state: EntryState::Active,
        notes: Vec::new(),
    };
    append_log_entry_to_path(path, date, &entry)
}

pub fn append_pending_entry(path: &Path, date: &str, time: &str, summary: &str) -> Result<()> {
    let entry = Entry {
        time: time.to_string(),
        summary: summary.to_string(),
        state: EntryState::Pending,
        notes: Vec::new(),
    };
    append_log_entry_to_path_without_demoting_active(path, date, &entry)
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

    let Some(target) = collect_unfinished_entries_from_contents(&contents)
        .last()
        .cloned()
    else {
        return Err(anyhow!("no unfinished entry found"));
    };

    mark_entry_done_with_contents(path, &contents, target.start, target.end)
}

pub fn mark_unfinished_entry_done(
    path: &Path,
    target: &UnfinishedEntry,
    _timestamp: &str,
) -> Result<()> {
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

pub fn mark_focus_entry_done(path: &Path, target: &FocusEntry, _timestamp: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let fresh_target = collect_focus_entries_from_contents(&contents)
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

pub fn mark_entry_done_by_transient_id(path: &Path, id: &str) -> Result<()> {
    let mut document = load_log_at_path(path)?;
    for day in &mut document.days {
        for entry in &mut day.entries {
            if entry.transient_id(&day.date) == id {
                if entry.state.is_done() {
                    return Ok(());
                }
                entry.state = EntryState::Done;
                save_log_to_path(path, &document)?;
                return Ok(());
            }
        }
    }

    Err(anyhow!("item changed or not found"))
}

pub fn apply_entry_state_updates(path: &Path, updates: &[(LogEntry, EntryState)]) -> Result<()> {
    if updates.is_empty() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let fresh_entries = collect_entries_from_contents(&contents);
    let mut replacements = Vec::with_capacity(updates.len());

    for (target, state) in updates {
        let fresh_target = fresh_entries
            .iter()
            .find(|entry| {
                entry.ordinal == target.ordinal
                    && entry.date == target.date
                    && entry.time == target.time
                    && entry.summary == target.summary
                    && entry.state == target.state
            })
            .ok_or_else(|| anyhow!("selected entry changed before it could be updated"))?;
        replacements.push((fresh_target.start, fresh_target.end, *state));
    }

    replacements.sort_by(|left, right| right.0.cmp(&left.0));

    let mut updated = contents;
    for (start, end, state) in replacements {
        let segment = &updated[start..end];
        let body = segment.trim_end_matches(['\r', '\n']);
        let ending = &segment[body.len()..];
        let updated_body = replace_checkbox(body, state);
        updated.replace_range(start..end, &(updated_body + ending));
    }

    fs::write(path, updated)
        .with_context(|| format!("failed to write log at {}", path.display()))?;
    Ok(())
}

pub fn delete_entry(path: &Path, target: &LogEntry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let mut document = load_log_at_path(path)?;
    let mut ordinal = 0usize;

    for day in &mut document.days {
        let mut index = 0usize;
        while index < day.entries.len() {
            let entry = &day.entries[index];
            if ordinal == target.ordinal
                && day.date == target.date
                && entry.time == target.time
                && entry.summary == target.summary
                && entry.state == target.state
            {
                day.entries.remove(index);
                document.days.retain(|day| !day.entries.is_empty());
                save_log_to_path(path, &document)?;
                return Ok(());
            }
            ordinal += 1;
            index += 1;
        }
    }

    Err(anyhow!("selected entry changed before it could be removed"))
}

pub fn delete_by_transient_id(path: &Path, id: &str) -> Result<()> {
    let mut document = load_log_at_path(path)?;

    for day in &mut document.days {
        if let Some(entry_index) = day
            .entries
            .iter()
            .position(|entry| entry.transient_id(&day.date) == id)
        {
            day.entries.remove(entry_index);
            document.days.retain(|day| !day.entries.is_empty());
            save_log_to_path(path, &document)?;
            return Ok(());
        }
    }

    for day in &mut document.days {
        for entry in &mut day.entries {
            if let Some(note_index) = entry.notes.iter().enumerate().find_map(|(index, note)| {
                (entry.transient_note_id(&day.date, index, note) == id).then_some(index)
            }) {
                entry.notes.remove(note_index);
                save_log_to_path(path, &document)?;
                return Ok(());
            }
        }
    }

    Err(anyhow!("item changed or not found"))
}

pub fn delete_removable_target(path: &Path, target: &RemovableTarget) -> Result<()> {
    let mut document = load_log_at_path(path)?;
    let mut entry_ordinal = 0usize;

    for day in &mut document.days {
        let mut entry_index = 0usize;
        while entry_index < day.entries.len() {
            let entry = &mut day.entries[entry_index];
            let entry_matches = entry_ordinal == target.entry_ordinal
                && day.date == target.date
                && entry.time == target.time
                && entry.summary == target.summary
                && entry.state == target.state;

            match target.kind {
                RemovableKind::Entry if entry_matches => {
                    day.entries.remove(entry_index);
                    document.days.retain(|day| !day.entries.is_empty());
                    save_log_to_path(path, &document)?;
                    return Ok(());
                }
                RemovableKind::Note if entry_matches => {
                    let Some(note_ordinal) = target.note_ordinal else {
                        break;
                    };
                    let Some(note_text) = target.note_text.as_deref() else {
                        break;
                    };
                    if note_ordinal < entry.notes.len() && entry.notes[note_ordinal] == note_text {
                        entry.notes.remove(note_ordinal);
                        save_log_to_path(path, &document)?;
                        return Ok(());
                    }
                    return Err(anyhow!("selected note changed before it could be removed"));
                }
                _ => {
                    entry_ordinal += 1;
                    entry_index += 1;
                }
            }
        }
    }

    let message = match target.kind {
        RemovableKind::Entry => "selected entry changed before it could be removed",
        RemovableKind::Note => "selected note changed before it could be removed",
    };
    Err(anyhow!(message))
}

pub fn edit_entry_summary(path: &Path, target: &LogEntry, summary: &str) -> Result<()> {
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
                && entry.state == target.state
        })
        .ok_or_else(|| anyhow!("selected entry changed before it could be edited"))?;

    edit_entry_summary_with_contents(path, &contents, &fresh_target, summary)
}

pub fn focus_entry(path: &Path, target: &FocusEntry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let fresh_target = collect_focus_entries_from_contents(&contents)
        .into_iter()
        .find(|entry| {
            entry.ordinal == target.ordinal
                && entry.date == target.date
                && entry.time == target.time
                && entry.summary == target.summary
        })
        .ok_or_else(|| anyhow!("selected entry changed before it could be focused"))?;

    if fresh_target.state.is_active() {
        return Ok(());
    }

    let demoted = demote_active_entries_in_contents(&contents);
    replace_entry_state_with_contents(
        path,
        &demoted,
        fresh_target.start,
        fresh_target.end,
        EntryState::Active,
    )
}

pub fn focus_latest_entry(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let entries = collect_entries_from_contents(&contents);
    if entries.is_empty() {
        return Err(anyhow!("no entry found"));
    }

    let Some(target) = entries
        .into_iter()
        .rev()
        .find(|entry| !entry.state.is_done())
    else {
        return Err(anyhow!("all entries are already done"));
    };

    if target.state.is_active() {
        return Ok(());
    }

    let demoted = demote_active_entries_in_contents(&contents);
    replace_entry_state_with_contents(path, &demoted, target.start, target.end, EntryState::Active)
}

pub fn append_note_to_latest_open_entry(path: &Path, note: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    let contents = read_log_contents(path)?;
    let Some(target) = collect_note_targets_from_contents(&contents) else {
        return Err(anyhow!("no note target found"));
    };

    let line_ending = newline_ending(&contents);
    let mut updated = String::with_capacity(contents.len() + note.len() + 8);
    updated.push_str(&contents[..target.insert_at]);
    updated.push_str("  - ");
    updated.push_str(note);
    updated.push_str(line_ending);
    updated.push_str(&contents[target.insert_at..]);

    fs::write(path, updated)
        .with_context(|| format!("failed to write log at {}", path.display()))?;
    Ok(())
}

pub fn append_note_by_transient_id(path: &Path, id: &str, note: &str) -> Result<()> {
    let mut document = load_log_at_path(path)?;

    for day in &mut document.days {
        for entry in &mut day.entries {
            if entry.transient_id(&day.date) == id {
                entry.notes.push(note.to_string());
                save_log_to_path(path, &document)?;
                return Ok(());
            }
        }
    }

    Err(anyhow!("item changed or not found"))
}

pub fn archive_done_entries_at_paths(log_path: &Path, archive_path: &Path) -> Result<()> {
    let log_document = load_log_at_path(log_path)?;
    let archive_document = load_log_at_path(archive_path)?;
    let (remaining_document, archived_document) = archive_documents(log_document, archive_document);

    save_log_to_path(log_path, &remaining_document)?;
    if archived_document.days.is_empty() {
        match fs::remove_file(archive_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to remove archive at {}", archive_path.display())
                });
            }
        }
    } else {
        save_log_to_path(archive_path, &archived_document)?;
    }
    Ok(())
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

fn archive_documents(
    log_document: LogDocument,
    mut archive_document: LogDocument,
) -> (LogDocument, LogDocument) {
    let mut remaining_days = Vec::new();

    for day in log_document.days {
        let mut archived_entries = Vec::new();
        let mut remaining_entries = Vec::new();

        for entry in day.entries {
            if entry.state.is_done() {
                archived_entries.push(entry);
            } else {
                remaining_entries.push(entry);
            }
        }

        if !archived_entries.is_empty() {
            if let Some(existing_day) = archive_document
                .days
                .iter_mut()
                .find(|existing| existing.date == day.date)
            {
                existing_day.entries.extend(archived_entries);
            } else {
                archive_document.days.push(DaySection {
                    date: day.date.clone(),
                    entries: archived_entries,
                });
            }
        }

        if !remaining_entries.is_empty() {
            remaining_days.push(DaySection {
                date: day.date,
                entries: remaining_entries,
            });
        }
    }

    archive_document.days.retain(|day| !day.entries.is_empty());

    (
        LogDocument {
            days: remaining_days,
        },
        archive_document,
    )
}

fn read_log_contents(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to read log at {}", path.display()))
        }
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
        } else if let Some(date) = current_date.as_ref()
            && let Some(entry) = parse_entry_line(line)
            && entry.state.is_pending()
        {
            entries.push(UnfinishedEntry {
                date: date.clone(),
                time: entry.time,
                summary: entry.summary,
                ordinal: entries.len(),
                start: line_start,
                end: line_end,
            });
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
        } else if let Some(date) = current_date.as_ref()
            && let Some(entry) = parse_entry_line(line)
        {
            entries.push(LogEntry {
                date: date.clone(),
                time: entry.time,
                summary: entry.summary,
                state: entry.state,
                ordinal: entries.len(),
                start: line_start,
                end: line_end,
            });
        }

        line_start = line_end;
    }

    entries
}

fn collect_removable_targets_from_contents(contents: &str) -> Vec<RemovableTarget> {
    let mut targets = Vec::new();
    let mut current_date: Option<String> = None;
    let mut current_entry: Option<(String, String, EntryState, usize)> = None;
    let mut entry_ordinal = 0usize;
    let mut note_ordinal = 0usize;

    for line in contents.lines() {
        if let Some(date) = parse_day_heading(line) {
            current_date = Some(date.to_string());
            current_entry = None;
            note_ordinal = 0;
        } else if line.starts_with("## ") {
            current_date = None;
            current_entry = None;
            note_ordinal = 0;
        } else if let Some(date) = current_date.as_ref() {
            if let Some(entry) = parse_entry_line(line) {
                targets.push(RemovableTarget {
                    kind: RemovableKind::Entry,
                    date: date.clone(),
                    time: entry.time.clone(),
                    summary: entry.summary.clone(),
                    state: entry.state,
                    entry_ordinal,
                    note_ordinal: None,
                    note_text: None,
                });
                current_entry = Some((
                    entry.time.clone(),
                    entry.summary.clone(),
                    entry.state,
                    entry_ordinal,
                ));
                entry_ordinal += 1;
                note_ordinal = 0;
            } else if let Some(note) = parse_note_line(line) {
                if let Some((time, summary, state, current_entry_ordinal)) = current_entry.as_ref()
                {
                    targets.push(RemovableTarget {
                        kind: RemovableKind::Note,
                        date: date.clone(),
                        time: time.clone(),
                        summary: summary.clone(),
                        state: *state,
                        entry_ordinal: *current_entry_ordinal,
                        note_ordinal: Some(note_ordinal),
                        note_text: Some(note.to_string()),
                    });
                    note_ordinal += 1;
                }
            } else {
                current_entry = None;
                note_ordinal = 0;
            }
        }
    }

    targets
}

fn collect_focus_entries_from_contents(contents: &str) -> Vec<FocusEntry> {
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
        } else if let Some(date) = current_date.as_ref()
            && let Some(entry) = parse_entry_line(line)
            && !entry.state.is_done()
        {
            entries.push(FocusEntry {
                date: date.clone(),
                time: entry.time,
                summary: entry.summary,
                state: entry.state,
                ordinal: entries.len(),
                start: line_start,
                end: line_end,
            });
        }

        line_start = line_end;
    }

    entries
}

#[derive(Debug, Clone, Copy)]
struct NoteTarget {
    state: EntryState,
    insert_at: usize,
}

fn collect_note_targets_from_contents(contents: &str) -> Option<NoteTarget> {
    let mut line_start = 0;
    let mut current_date: Option<String> = None;
    let mut current_target: Option<NoteTarget> = None;
    let mut active_target: Option<NoteTarget> = None;
    let mut last_pending_target: Option<NoteTarget> = None;

    for segment in contents.split_inclusive('\n') {
        let line_end = line_start + segment.len();
        let line = segment.trim_end();

        if let Some(date) = parse_day_heading(line) {
            current_date = Some(date.to_string());
            current_target = None;
        } else if line.starts_with("## ") {
            current_date = None;
            current_target = None;
        } else if current_date.is_some() {
            if let Some(entry) = parse_entry_line(line) {
                let target = NoteTarget {
                    state: entry.state,
                    insert_at: line_end,
                };
                current_target = Some(target);
                if entry.state.is_active() {
                    active_target = Some(target);
                } else if entry.state.is_pending() {
                    last_pending_target = Some(target);
                }
            } else if parse_note_line(line).is_some() {
                if let Some(target) = current_target.as_mut() {
                    target.insert_at = line_end;
                    if target.state.is_active() {
                        active_target = Some(*target);
                    } else if target.state.is_pending() {
                        last_pending_target = Some(*target);
                    }
                }
            } else {
                current_target = None;
            }
        }

        line_start = line_end;
    }

    active_target.or(last_pending_target)
}

fn mark_entry_done_with_contents(
    path: &Path,
    contents: &str,
    start: usize,
    end: usize,
) -> Result<()> {
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

    fs::write(path, updated)
        .with_context(|| format!("failed to write log at {}", path.display()))?;
    Ok(())
}

fn edit_entry_summary_with_contents(
    path: &Path,
    contents: &str,
    target: &LogEntry,
    summary: &str,
) -> Result<()> {
    let mut updated = String::with_capacity(contents.len() + summary.len());
    updated.push_str(&contents[..target.start]);
    let segment = &contents[target.start..target.end];
    let body = segment.trim_end_matches(['\r', '\n']);
    let ending = &segment[body.len()..];
    let entry = parse_entry_line(body).ok_or_else(|| anyhow!("failed to parse target entry"))?;
    let updated_body = format_entry(&Entry {
        time: entry.time,
        summary: summary.to_string(),
        state: entry.state,
        notes: Vec::new(),
    });
    updated.push_str(&updated_body);
    updated.push_str(ending);
    updated.push_str(&contents[target.end..]);

    fs::write(path, updated)
        .with_context(|| format!("failed to write log at {}", path.display()))?;
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
        Err(error) => {
            Err(error).with_context(|| format!("failed to read log at {}", path.display()))
        }
    }
}

fn append_log_entry_to_path_without_demoting_active(
    path: &Path,
    date: &str,
    entry: &Entry,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory at {}", parent.display()))?;
    }

    match fs::read_to_string(path) {
        Ok(contents) => append_to_existing_log(path, &contents, date, entry),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            write_new_log(path, date, entry)
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to read log at {}", path.display()))
        }
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
        let needs_separator =
            insertion_point > 0 && !contents[..insertion_point].ends_with(line_ending);
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
    if updated.ends_with(line_ending) && !updated.ends_with(&format!("{line_ending}{line_ending}"))
    {
        updated.push_str(line_ending);
    }
    updated.push_str(
        &format_day_section(&DaySection {
            date: date.to_string(),
            entries: vec![entry.clone()],
        })
        .replace('\n', line_ending),
    );

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
        .find(|entry| {
            entry_line_state(contents, entry.start, entry.end) == Some(EntryState::Active)
        })
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
    if let Some(rest) = line.strip_prefix("- [>] ") {
        format!("- {} {}", state.checkbox(), rest)
    } else if let Some(rest) = line.strip_prefix("- [x] ") {
        format!("- {} {}", state.checkbox(), rest)
    } else if let Some(rest) = line.strip_prefix("- [ ] ") {
        format!("- {} {}", state.checkbox(), rest)
    } else {
        line.to_string()
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

        let loaded = load_log_at_path(&path).unwrap();

        assert_eq!(loaded, LogDocument::default());
    }

    #[test]
    fn load_log_returns_context_for_other_read_errors() {
        let dir = unique_temp_dir("read-error");
        fs::create_dir_all(&dir).unwrap();

        let error = load_log_at_path(&dir).unwrap_err();
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
