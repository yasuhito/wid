use std::io::{self, Write};

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType};

use crate::log::model::PickerItem;

pub trait Picker {
    fn pick<T: PickerItem>(&mut self, entries: &[T]) -> Result<Option<usize>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerOutcome {
    Continue,
    Confirmed(usize),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct PickerState {
    selected: usize,
    len: usize,
}

impl PickerState {
    pub fn new(len: usize) -> Self {
        Self { selected: 0, len }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> PickerOutcome {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < self.len {
                    self.selected += 1;
                }
                PickerOutcome::Continue
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                PickerOutcome::Continue
            }
            KeyCode::Enter => PickerOutcome::Confirmed(self.selected),
            KeyCode::Char('q') | KeyCode::Esc => PickerOutcome::Cancelled,
            _ => PickerOutcome::Continue,
        }
    }
}

pub struct TerminalPicker;

impl Picker for TerminalPicker {
    fn pick<T: PickerItem>(&mut self, entries: &[T]) -> Result<Option<usize>> {
        if entries.is_empty() {
            return Ok(None);
        }

        terminal::enable_raw_mode()?;
        let _guard = RawModeGuard;
        let mut state = PickerState::new(entries.len());
        let mut stdout = io::stdout();

        loop {
            render(&mut stdout, entries, state.selected())?;
            stdout.flush()?;

            if let Event::Key(key) = event::read()? {
                match state.handle_key(key) {
                    PickerOutcome::Continue => {}
                    PickerOutcome::Confirmed(index) => {
                        write_crlf(&mut stdout)?;
                        stdout.flush()?;
                        return Ok(Some(index));
                    }
                    PickerOutcome::Cancelled => {
                        write_crlf(&mut stdout)?;
                        stdout.flush()?;
                        return Ok(None);
                    }
                }
            }
        }
    }
}

fn render<T: PickerItem>(stdout: &mut impl Write, entries: &[T], selected: usize) -> Result<()> {
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    write_line(stdout, "Select an entry:")?;

    for (index, entry) in entries.iter().enumerate() {
        let prefix = if index == selected { ">" } else { " " };
        let label = entry.display_label();
        write_line(stdout, &format!("{prefix} {label}"))?;
    }

    write_crlf(stdout)?;
    write_line(stdout, "j/Down next, k/Up previous, Enter confirm, q/Esc cancel")?;
    Ok(())
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

#[allow(dead_code)]
fn _accept_modifier(_: KeyModifiers) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_uses_crlf_line_endings_in_raw_mode_friendly_output() {
        let mut output = Vec::new();
        let entries = vec![crate::log::model::UnfinishedEntry {
            date: "2026-03-24".into(),
            time: "11:32".into(),
            summary: "spaced entry".into(),
            ordinal: 0,
            start: 0,
            end: 0,
        }];

        render(&mut output, &entries, 0).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Select an entry:\r\n"));
        assert!(rendered.contains("> 2026-03-24 11:32 spaced entry\r\n"));
        assert!(rendered.contains("\r\nj/Down next, k/Up previous, Enter confirm, q/Esc cancel\r\n"));
    }
}
