use super::model::{DaySection, Entry, LogDocument};

pub fn parse_log(input: &str) -> anyhow::Result<LogDocument> {
    let mut document = LogDocument::default();
    let mut current_day: Option<DaySection> = None;

    for raw_line in input.lines() {
        let line = raw_line.trim_end();

        if let Some(date) = parse_day_heading(line) {
            if let Some(day) = current_day.take() {
                document.days.push(day);
            }

            current_day = Some(DaySection {
                date,
                entries: Vec::new(),
            });
            continue;
        }

        if let Some(entry) = parse_entry(line) {
            if let Some(day) = current_day.as_mut() {
                day.entries.push(entry);
            }
        }
    }

    if let Some(day) = current_day {
        document.days.push(day);
    }

    Ok(document)
}

fn parse_day_heading(line: &str) -> Option<String> {
    let date = line.strip_prefix("## ")?;
    if is_date_like(date) {
        Some(date.to_string())
    } else {
        None
    }
}

fn parse_entry(line: &str) -> Option<Entry> {
    let rest = line.strip_prefix("- ")?;
    let (time, summary) = rest.split_once(' ')?;
    if !is_time_like(time) {
        return None;
    }

    if summary.is_empty() || summary.starts_with(' ') {
        return None;
    }

    Some(Entry {
        time: time.to_string(),
        summary: summary.to_string(),
    })
}

fn is_date_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn is_time_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 5
        && bytes[0..2].iter().all(u8::is_ascii_digit)
        && bytes[2] == b':'
        && bytes[3..5].iter().all(u8::is_ascii_digit)
}
