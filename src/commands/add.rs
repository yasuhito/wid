use anyhow::Result;

use crate::commands::now::{current_local_date_time, resolve_summary};
use crate::log::{paths::default_log_path, store::append_pending_entry};

pub fn run(text: Vec<String>) -> Result<()> {
    let summary = resolve_summary(text)?;
    let (date, time) = current_local_date_time()?;
    let path = default_log_path()?;
    append_pending_entry(&path, &date, &time, &summary)
}
