#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryState {
    #[default]
    Pending,
    Active,
    Done,
}

impl EntryState {
    pub fn checkbox(self) -> &'static str {
        match self {
            Self::Pending => "[ ]",
            Self::Active => "[>]",
            Self::Done => "[x]",
        }
    }

    pub fn is_pending(self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn is_done(self) -> bool {
        matches!(self, Self::Done)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Done => "done",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    pub time: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub state: EntryState,
    pub notes: Vec<String>,
}

impl Entry {
    pub fn transient_id(&self, date: &str) -> String {
        let digest = md5::compute(format!(
            "{date}\n{}\n{}\n{}",
            self.time,
            self.state.as_str(),
            self.summary
        ));
        format!("{digest:x}").chars().take(12).collect()
    }

    pub fn transient_note_id(&self, date: &str, note: &str) -> String {
        let entry_id = self.transient_id(date);
        let digest = md5::compute(format!("{entry_id}\n{note}"));
        format!("note_{digest:x}").chars().take(17).collect()
    }
}

pub fn format_summary_with_tags(summary: &str, tags: &[String]) -> String {
    if tags.is_empty() {
        return summary.to_string();
    }

    format!(
        "{summary} {}",
        tags.iter()
            .map(|tag| format!("@{tag}"))
            .collect::<Vec<_>>()
            .join(" ")
    )
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

    fn line_count(&self) -> usize {
        1
    }

    fn delete_prompt(&self) -> &'static str {
        "Delete selected entry? [y/N]"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnfinishedEntry {
    pub date: String,
    pub time: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
}

impl UnfinishedEntry {
    pub fn display_label(&self) -> String {
        format!(
            "{} {} {}",
            self.date,
            self.time,
            format_summary_with_tags(&self.summary, &self.tags)
        )
    }
}

impl PickerItem for UnfinishedEntry {
    fn display_label(&self) -> String {
        self.display_label()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusEntry {
    pub date: String,
    pub time: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub state: EntryState,
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
}

impl FocusEntry {
    pub fn display_label(&self) -> String {
        format!(
            "{} {} {} {}",
            self.date,
            self.state.checkbox(),
            self.time,
            format_summary_with_tags(&self.summary, &self.tags)
        )
    }
}

impl PickerItem for FocusEntry {
    fn display_label(&self) -> String {
        self.display_label()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub date: String,
    pub time: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub notes: Vec<String>,
    pub state: EntryState,
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
}

impl LogEntry {
    pub fn transient_id(&self) -> String {
        let digest = md5::compute(format!(
            "{}\n{}\n{}\n{}",
            self.date,
            self.time,
            self.state.as_str(),
            self.summary
        ));
        format!("{digest:x}").chars().take(12).collect()
    }

    pub fn display_label(&self) -> String {
        self.display_label_with_state(self.state)
    }

    pub fn display_label_with_state(&self, state: EntryState) -> String {
        let mut lines = vec![format!(
            "{} {} {} {}",
            self.date,
            state.checkbox(),
            self.time,
            format_summary_with_tags(&self.summary, &self.tags)
        )];
        lines.extend(self.notes.iter().map(|note| format!("  📝 {note}")));
        lines.join("\n")
    }

    pub fn display_line_count(&self) -> usize {
        1 + self.notes.len()
    }
}

impl PickerItem for LogEntry {
    fn display_label(&self) -> String {
        self.display_label()
    }

    fn line_count(&self) -> usize {
        self.display_line_count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovableKind {
    Entry,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovableTarget {
    pub kind: RemovableKind,
    pub date: String,
    pub time: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub state: EntryState,
    pub entry_ordinal: usize,
    pub note_ordinal: Option<usize>,
    pub note_text: Option<String>,
}

impl RemovableTarget {
    pub fn display_label(&self) -> String {
        match self.kind {
            RemovableKind::Entry => format!(
                "{} {} {} {}",
                self.date,
                self.state.checkbox(),
                self.time,
                format_summary_with_tags(&self.summary, &self.tags)
            ),
            RemovableKind::Note => {
                format!("  📝 {}", self.note_text.as_deref().unwrap_or_default())
            }
        }
    }
}

impl PickerItem for RemovableTarget {
    fn display_label(&self) -> String {
        self.display_label()
    }

    fn delete_prompt(&self) -> &'static str {
        match self.kind {
            RemovableKind::Entry => "Delete selected entry? [y/N]",
            RemovableKind::Note => "Delete selected note? [y/N]",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entry_display_label_shows_summary_verbatim() {
        let done_entry = LogEntry {
            date: "2026-03-25".into(),
            time: "09:15".into(),
            summary: "completed work".into(),
            tags: Vec::new(),
            notes: vec!["left a note".into()],
            state: EntryState::Done,
            ordinal: 0,
            start: 0,
            end: 0,
        };
        let non_done_entry = LogEntry {
            date: "2026-03-25".into(),
            time: "09:16".into(),
            summary: "investigate [x]".into(),
            tags: Vec::new(),
            notes: Vec::new(),
            state: EntryState::Pending,
            ordinal: 1,
            start: 0,
            end: 0,
        };

        assert_eq!(
            done_entry.display_label(),
            "2026-03-25 [x] 09:15 completed work\n  📝 left a note"
        );
        assert_eq!(
            non_done_entry.display_label(),
            "2026-03-25 [ ] 09:16 investigate [x]"
        );
    }

    #[test]
    fn format_summary_with_tags_appends_at_tags() {
        assert_eq!(
            format_summary_with_tags("fix flaky CI", &["wid".into(), "agent".into()]),
            "fix flaky CI @wid @agent"
        );
    }
}
