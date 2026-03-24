use std::io::{self, Write};

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType};

use crate::log::model::UnfinishedEntry;

pub trait Picker {
    fn pick(&mut self, entries: &[UnfinishedEntry]) -> Result<Option<usize>>;
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
    fn pick(&mut self, entries: &[UnfinishedEntry]) -> Result<Option<usize>> {
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
                        writeln!(stdout)?;
                        stdout.flush()?;
                        return Ok(Some(index));
                    }
                    PickerOutcome::Cancelled => {
                        writeln!(stdout)?;
                        stdout.flush()?;
                        return Ok(None);
                    }
                }
            }
        }
    }
}

fn render(stdout: &mut impl Write, entries: &[UnfinishedEntry], selected: usize) -> Result<()> {
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    writeln!(stdout, "Select an unfinished entry:")?;

    for (index, entry) in entries.iter().enumerate() {
        let prefix = if index == selected { ">" } else { " " };
        let label = entry.display_label();
        writeln!(stdout, "{prefix} {label}")?;
    }

    writeln!(stdout, "")?;
    writeln!(stdout, "j/Down next, k/Up previous, Enter confirm, q/Esc cancel")?;
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
