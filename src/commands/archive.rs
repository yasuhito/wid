use std::fs;

use anyhow::Result;

use crate::commands::show::print_log_if_changed;
use crate::log::{paths::default_log_path, store::archive_done_entries};

pub fn run() -> Result<()> {
    let log_path = default_log_path()?;
    let before = fs::read_to_string(&log_path).unwrap_or_default();
    archive_done_entries()?;
    print_log_if_changed(&log_path, &before)
}
