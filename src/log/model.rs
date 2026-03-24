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
