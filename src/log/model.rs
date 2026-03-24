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
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    pub time: String,
    pub summary: String,
    pub state: EntryState,
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
pub struct FocusEntry {
    pub date: String,
    pub time: String,
    pub summary: String,
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
            self.summary
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
    pub state: EntryState,
    pub ordinal: usize,
    pub start: usize,
    pub end: usize,
}

impl LogEntry {
    pub fn display_label(&self) -> String {
        format!(
            "{} {} {} {}",
            self.date,
            self.state.checkbox(),
            self.time,
            self.summary
        )
    }
}

impl PickerItem for LogEntry {
    fn display_label(&self) -> String {
        self.display_label()
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
            state: EntryState::Done,
            ordinal: 0,
            start: 0,
            end: 0,
        };
        let non_done_entry = LogEntry {
            date: "2026-03-25".into(),
            time: "09:16".into(),
            summary: "investigate [x]".into(),
            state: EntryState::Pending,
            ordinal: 1,
            start: 0,
            end: 0,
        };

        assert_eq!(done_entry.display_label(), "2026-03-25 [x] 09:15 completed work");
        assert_eq!(non_done_entry.display_label(), "2026-03-25 [ ] 09:16 investigate [x]");
    }
}
