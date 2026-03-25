use std::fs;

use anyhow::Result;

use crate::commands::now::{current_local_date_time, resolve_summary};
use crate::commands::show::print_log_if_changed;
use crate::log::{paths::default_log_path, store::append_pending_entry};

pub fn run(text: Vec<String>) -> Result<()> {
    let summary = resolve_summary(text)?;
    let (date, time) = current_local_date_time()?;
    let path = default_log_path()?;
    let before = fs::read_to_string(&path).unwrap_or_default();
    append_pending_entry(&path, &date, &time, &summary)?;
    print_log_if_changed(&path, &before)
}
