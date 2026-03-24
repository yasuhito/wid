use super::model::{DaySection, Entry, LogDocument};

pub fn format_log_header() -> &'static str {
    "# wid log\n\n"
}

pub fn format_entry(entry: &Entry) -> String {
    format!("- {} {}", entry.time, entry.summary)
}

pub fn format_day_heading(date: &str) -> String {
    format!("## {date}\n\n")
}

pub fn format_day_section(day: &DaySection) -> String {
    let mut output = format_day_heading(&day.date);

    for entry in &day.entries {
        output.push_str(&format_entry(entry));
        output.push('\n');
    }

    output
}

pub fn format_log(document: &LogDocument) -> String {
    let mut output = String::from(format_log_header());

    for (index, day) in document.days.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }

        output.push_str(&format_day_section(day));
    }

    output
}
