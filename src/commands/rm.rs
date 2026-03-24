use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use std::io::{self, Write};
use std::path::Path;

use crate::interactive::done_picker::{Picker, TerminalPicker};
use crate::log::store;

pub fn run(interactive: bool) -> Result<()> {
    if !interactive {
        return Err(anyhow!("rm requires -i"));
    }

    let path = crate::log::paths::default_log_path()?;
    let mut picker = TerminalPicker;
    let mut confirmer = TerminalConfirm;
    run_interactive_at_path(&path, &mut picker, &mut confirmer)
}

pub trait Confirm {
    fn confirm(&mut self) -> Result<bool>;
}

pub struct TerminalConfirm;

impl Confirm for TerminalConfirm {
    fn confirm(&mut self) -> Result<bool> {
        terminal::enable_raw_mode()?;
        let _guard = RawModeGuard;
        let mut stdout = io::stdout();

        write_line(&mut stdout, "Delete selected entry? [y/N]")?;
        stdout.flush()?;

        loop {
            let Some(delete) = confirm_decision_from_event(event::read()?) else {
                continue;
            };
            write_crlf(&mut stdout)?;
            stdout.flush()?;
            return Ok(delete);
        }
    }
}

pub fn run_interactive_at_path(
    path: &Path,
    picker: &mut impl Picker,
    confirmer: &mut impl Confirm,
) -> Result<()> {
    let entries = store::collect_entries(path)?;
    if entries.is_empty() {
        return Err(anyhow!("no entries found"));
    }

    let display_entries: Vec<_> = entries.into_iter().rev().collect();
    let Some(index) = picker.pick(&display_entries)? else {
        return Ok(());
    };

    let Some(target) = display_entries.get(index) else {
        return Err(anyhow!("invalid selection"));
    };

    if confirmer.confirm()? {
        store::delete_entry(path, target)
    } else {
        Ok(())
    }
}

fn is_confirm_delete_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('y'))
}

fn confirm_decision_from_event(event: Event) -> Option<bool> {
    match event {
        Event::Key(key) => Some(is_confirm_delete_key(key.code)),
        _ => None,
    }
}

fn write_line(stdout: &mut impl Write, line: &str) -> Result<()> {
    stdout.write_all(line.as_bytes())?;
    write_crlf(stdout)
}

fn write_crlf(stdout: &mut impl Write) -> Result<()> {
    stdout.write_all(b"\r\n")?;
    Ok(())
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_only_accepts_lowercase_y() {
        assert!(is_confirm_delete_key(KeyCode::Char('y')));
        assert!(!is_confirm_delete_key(KeyCode::Char('n')));
        assert!(!is_confirm_delete_key(KeyCode::Char('q')));
        assert!(!is_confirm_delete_key(KeyCode::Esc));
        assert!(!is_confirm_delete_key(KeyCode::Enter));
        assert!(!is_confirm_delete_key(KeyCode::Char('Y')));
    }

    #[test]
    fn confirmation_ignores_non_key_events() {
        assert_eq!(confirm_decision_from_event(Event::Resize(80, 24)), None);
        assert_eq!(confirm_decision_from_event(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char('y')))), Some(true));
        assert_eq!(confirm_decision_from_event(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char('n')))), Some(false));
    }
}
