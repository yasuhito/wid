use super::model::{DaySection, Entry, LogDocument};

pub fn format_entry(entry: &Entry) -> String {
    let checkbox = entry.state.checkbox();
    format!("- {checkbox} {} {}", entry.time, entry.summary)
}

pub fn format_day_heading(date: &str) -> String {
    format!("## {date}\n\n")
}

pub fn format_day_section(day: &DaySection) -> String {
    if day.entries.is_empty() {
        return String::new();
    }

    let mut output = format_day_heading(&day.date);

    for entry in &day.entries {
        output.push_str(&format_entry(entry));
        output.push('\n');
    }

    output
}

pub fn format_log(document: &LogDocument) -> String {
    let mut output = String::new();
    let non_empty_days: Vec<_> = document.days.iter().filter(|day| !day.entries.is_empty()).collect();

    for (index, day) in non_empty_days.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }

        output.push_str(&format_day_section(day));
    }

    output
}
