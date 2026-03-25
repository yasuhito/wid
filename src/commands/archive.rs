use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::{Result, anyhow};

use crate::commands::show::print_log_if_changed;
use crate::log::{
    paths::default_log_path,
    store::{archive_done_entries, load_log},
};

pub fn run(yes: bool) -> Result<()> {
    let log_path = default_log_path()?;
    let before = fs::read_to_string(&log_path).unwrap_or_default();
    let done_count = load_log()?
        .days
        .iter()
        .flat_map(|day| day.entries.iter())
        .filter(|entry| entry.state.is_done())
        .count();

    if done_count > 0 && !yes {
        let stdin = io::stdin();
        if !stdin.is_terminal() {
            return Err(anyhow!("archive requires confirmation; use --yes"));
        }

        let mut reader = stdin.lock();
        let mut writer = io::stdout();
        if !confirm_archive(done_count, &mut reader, &mut writer)? {
            return Ok(());
        }
    }

    archive_done_entries()?;
    print_log_if_changed(&log_path, &before)
}

pub fn confirm_archive(
    done_count: usize,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<bool> {
    let noun = if done_count == 1 { "entry" } else { "entries" };
    write!(
        writer,
        "Archive {done_count} done {noun} to archive.md? [y/N] "
    )?;
    writer.flush()?;

    let mut input = String::new();
    reader.read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y"))
}
