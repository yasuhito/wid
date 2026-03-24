#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    pub time: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DaySection {
    pub date: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogDocument {
    pub days: Vec<DaySection>,
}

pub trait PickerItem {
    fn display_label(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnfinishedEntry {
    pub date: String,
    pub time: String,
    pub summary: String,
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
}

impl UnfinishedEntry {
    pub fn display_label(&self) -> String {
        format!("{} {} {}", self.date, self.time, self.summary)
    }
}

impl PickerItem for UnfinishedEntry {
    fn display_label(&self) -> String {
        self.display_label()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub date: String,
    pub time: String,
    pub summary: String,
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
}

impl LogEntry {
    pub fn display_label(&self) -> String {
        format!("{} {} {}", self.date, self.time, summary_without_done_tag(&self.summary))
    }
}

impl PickerItem for LogEntry {
    fn display_label(&self) -> String {
        self.display_label()
    }
}

fn summary_without_done_tag(summary: &str) -> &str {
    let Some((body, timestamp)) = summary.rsplit_once(" @done(") else {
        return summary;
    };

    if is_done_timestamp(timestamp) {
        body
    } else {
        summary
    }
}

fn is_done_timestamp(value: &str) -> bool {
    let Some(timestamp) = value.strip_suffix(')') else {
        return false;
    };
    let bytes = timestamp.as_bytes();
    bytes.len() == 16
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[10] == b' '
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[13] == b':'
        && bytes[14..16].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entry_display_label_only_hides_valid_done_tags() {
        let done_entry = LogEntry {
            date: "2026-03-25".into(),
            time: "09:15".into(),
            summary: "completed work @done(2026-03-25 09:20)".into(),
            ordinal: 0,
            start: 0,
            end: 0,
        };
        let non_done_entry = LogEntry {
            date: "2026-03-25".into(),
            time: "09:16".into(),
            summary: "investigate @done(tbd)".into(),
            ordinal: 1,
            start: 0,
            end: 0,
        };

        assert_eq!(done_entry.display_label(), "2026-03-25 09:15 completed work");
        assert_eq!(non_done_entry.display_label(), "2026-03-25 09:16 investigate @done(tbd)");
    }
}
