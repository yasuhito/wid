use std::io;

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Clear as ClearWidget;
use ratatui::widgets::{List, ListItem, ListState, Paragraph, StatefulWidget, Widget};
use ratatui::{Frame, Terminal};

use crate::commands::show::{render_day_heading, render_day_separator, render_entry_summary};
use crate::log::model::{EntryState, GroupedPickerItem, LogEntry, PickerItem};

pub trait Picker {
    fn pick<T: PickerItem + GroupedPickerItem>(&mut self, entries: &[T]) -> Result<Option<usize>>;

    fn pick_with_selected<T: PickerItem + GroupedPickerItem>(
        &mut self,
        entries: &[T],
        _selected: usize,
    ) -> Result<Option<usize>> {
        self.pick(entries)
    }
}

pub trait DoneStatePicker {
    fn pick_done_states(
        &mut self,
        entries: &[LogEntry],
        selected: usize,
    ) -> Result<Option<Vec<EntryState>>>;
}

pub trait GroupedLogEntryPicker {
    fn pick_grouped_entries(
        &mut self,
        entries: &[LogEntry],
        selected: usize,
    ) -> Result<Option<usize>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerOutcome {
    Continue,
    Confirmed(usize),
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DonePickerOutcome {
    Continue,
    Confirmed(Vec<EntryState>),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct PickerState {
    selected: usize,
    len: usize,
}

#[derive(Debug, Clone)]
pub struct DonePickerState {
    selected: usize,
    states: Vec<EntryState>,
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

impl DonePickerState {
    pub fn new(states: Vec<EntryState>) -> Self {
        Self {
            selected: 0,
            states,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn states(&self) -> &[EntryState] {
        &self.states
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DonePickerOutcome {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < self.states.len() {
                    self.selected += 1;
                }
                DonePickerOutcome::Continue
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                DonePickerOutcome::Continue
            }
            KeyCode::Char(' ') => {
                if let Some(state) = self.states.get_mut(self.selected) {
                    *state = match *state {
                        EntryState::Pending => EntryState::Done,
                        EntryState::Active => EntryState::Done,
                        EntryState::Done => EntryState::Pending,
                    };
                }
                DonePickerOutcome::Continue
            }
            KeyCode::Enter => DonePickerOutcome::Confirmed(self.states.clone()),
            KeyCode::Char('q') | KeyCode::Esc => DonePickerOutcome::Cancelled,
            _ => DonePickerOutcome::Continue,
        }
    }
}

pub struct TerminalPicker;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupedPickerRow {
    label: String,
    selectable_entry_index: Option<usize>,
    line_count: usize,
}

impl Picker for TerminalPicker {
    fn pick<T: PickerItem + GroupedPickerItem>(&mut self, entries: &[T]) -> Result<Option<usize>> {
        self.pick_with_selected(entries, 0)
    }

    fn pick_with_selected<T: PickerItem + GroupedPickerItem>(
        &mut self,
        entries: &[T],
        selected: usize,
    ) -> Result<Option<usize>> {
        if entries.is_empty() {
            return Ok(None);
        }

        let rows = build_grouped_rows(entries);
        let anchor_row = cursor::position()?.1;
        let clear_area = panel_area(
            Rect::new(0, 0, terminal::size()?.0, terminal::size()?.1),
            anchor_row,
            grouped_picker_content_lines(&rows),
            PickerMode::Browse,
        );
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, cursor::Hide)?;
        let _guard = TerminalGuard { clear_area };

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = PickerState::new(entries.len());
        state.selected = selected.min(entries.len().saturating_sub(1));

        loop {
            terminal.draw(|frame| render_grouped_frame(frame, &rows, state.selected(), anchor_row))?;

            if let Event::Key(key) = event::read()? {
                match state.handle_key(key) {
                    PickerOutcome::Continue => {}
                    PickerOutcome::Confirmed(index) => return Ok(Some(index)),
                    PickerOutcome::Cancelled => return Ok(None),
                }
            }
        }
    }
}

impl GroupedLogEntryPicker for TerminalPicker {
    fn pick_grouped_entries(
        &mut self,
        entries: &[LogEntry],
        selected: usize,
    ) -> Result<Option<usize>> {
        if entries.is_empty() {
            return Ok(None);
        }

        let rows = build_grouped_rows(entries);
        let anchor_row = cursor::position()?.1;
        let clear_area = panel_area(
            Rect::new(0, 0, terminal::size()?.0, terminal::size()?.1),
            anchor_row,
            grouped_picker_content_lines(&rows),
            PickerMode::Browse,
        );
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, cursor::Hide)?;
        let _guard = TerminalGuard { clear_area };

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = PickerState::new(entries.len());
        state.selected = selected.min(entries.len().saturating_sub(1));

        loop {
            terminal.draw(|frame| {
                render_grouped_frame(frame, &rows, state.selected(), anchor_row)
            })?;

            if let Event::Key(key) = event::read()? {
                match state.handle_key(key) {
                    PickerOutcome::Continue => {}
                    PickerOutcome::Confirmed(index) => return Ok(Some(index)),
                    PickerOutcome::Cancelled => return Ok(None),
                }
            }
        }
    }
}

impl TerminalPicker {
    pub fn pick_done_states(
        &mut self,
        entries: &[LogEntry],
        selected: usize,
    ) -> Result<Option<Vec<EntryState>>> {
        if entries.is_empty() {
            return Ok(None);
        }

        let anchor_row = cursor::position()?.1;
        let initial_states: Vec<EntryState> = entries.iter().map(|entry| entry.state).collect();
        let rows = build_grouped_done_rows(entries, &initial_states);
        let clear_area = panel_area(
            Rect::new(0, 0, terminal::size()?.0, terminal::size()?.1),
            anchor_row,
            grouped_picker_content_lines(&rows),
            PickerMode::Browse,
        );
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, cursor::Hide)?;
        let _guard = TerminalGuard { clear_area };

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = DonePickerState::new(initial_states);
        state.selected = selected.min(entries.len().saturating_sub(1));

        loop {
            terminal.draw(|frame| {
                render_done_frame(frame, entries, state.states(), state.selected(), anchor_row)
            })?;

            if let Event::Key(key) = event::read()? {
                match state.handle_key(key) {
                    DonePickerOutcome::Continue => {}
                    DonePickerOutcome::Confirmed(states) => return Ok(Some(states)),
                    DonePickerOutcome::Cancelled => return Ok(None),
                }
            }
        }
    }

    pub fn pick_for_delete<T: PickerItem + GroupedPickerItem>(
        &mut self,
        entries: &[T],
    ) -> Result<Option<usize>> {
        if entries.is_empty() {
            return Ok(None);
        }

        let anchor_row = cursor::position()?.1;
        let terminal_size = terminal::size()?;
        let rows = build_grouped_rows(entries);
        let clear_area = panel_area(
            Rect::new(0, 0, terminal_size.0, terminal_size.1),
            anchor_row,
            grouped_picker_content_lines(&rows),
            PickerMode::ConfirmDelete,
        );
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, cursor::Hide)?;
        let _guard = TerminalGuard { clear_area };

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut state = PickerState::new(entries.len());
        let mut mode = PickerMode::Browse;

        loop {
            terminal.draw(|frame| {
                render_grouped_delete_frame(frame, &rows, entries, state.selected(), mode, anchor_row)
            })?;

            if let Event::Key(key) = event::read()? {
                match mode {
                    PickerMode::Browse => match state.handle_key(key) {
                        PickerOutcome::Continue => {}
                        PickerOutcome::Confirmed(_) => mode = PickerMode::ConfirmDelete,
                        PickerOutcome::Cancelled => return Ok(None),
                    },
                    PickerMode::ConfirmDelete => {
                        if confirm_delete_key(key) {
                            return Ok(Some(state.selected()));
                        }
                        mode = PickerMode::Browse;
                    }
                }
            }
        }
    }
}

impl DoneStatePicker for TerminalPicker {
    fn pick_done_states(
        &mut self,
        entries: &[LogEntry],
        selected: usize,
    ) -> Result<Option<Vec<EntryState>>> {
        TerminalPicker::pick_done_states(self, entries, selected)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerMode {
    Browse,
    ConfirmDelete,
}

#[derive(Debug, Clone, Copy)]
struct PickerTheme {
    title: Style,
    normal: Style,
    selected: Style,
    accent: Style,
    footer: Style,
}

impl PickerTheme {
    fn omarchy() -> Self {
        Self {
            title: Style::default().fg(Color::Rgb(160, 167, 191)),
            normal: Style::default().fg(Color::Rgb(188, 194, 216)),
            selected: Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(191, 183, 155))
                .add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::Rgb(217, 48, 95)),
            footer: Style::default().fg(Color::Rgb(132, 139, 166)),
        }
    }
}

fn render_done_frame(
    frame: &mut Frame,
    entries: &[LogEntry],
    states: &[EntryState],
    selected: usize,
    anchor_row: u16,
) {
    let rows = build_grouped_done_rows(entries, states);
    let area = panel_area(
        frame.area(),
        anchor_row,
        grouped_picker_content_lines(&rows),
        PickerMode::Browse,
    );
    ClearWidget.render(area, frame.buffer_mut());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let theme = PickerTheme::omarchy();

    Paragraph::new("Toggle done state:")
        .style(theme.title)
        .render(chunks[0], frame.buffer_mut());

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(chunks[1]);

    render_grouped_list(frame, body[0], &rows, selected, theme);
    Paragraph::new("j/Down next, k/Up previous, Space toggle, Enter confirm, q/Esc cancel")
        .style(theme.footer)
        .render(body[1], frame.buffer_mut());
}

fn render_grouped_delete_frame<T: PickerItem + GroupedPickerItem>(
    frame: &mut Frame,
    rows: &[GroupedPickerRow],
    entries: &[T],
    selected_entry: usize,
    mode: PickerMode,
    anchor_row: u16,
) {
    let area = panel_area(
        frame.area(),
        anchor_row,
        grouped_picker_content_lines(rows),
        mode,
    );
    ClearWidget.render(area, frame.buffer_mut());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let theme = PickerTheme::omarchy();

    Paragraph::new("Select an entry:")
        .style(theme.title)
        .render(chunks[0], frame.buffer_mut());

    match mode {
        PickerMode::Browse => {
            let body = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(chunks[1]);
            render_grouped_list(frame, body[0], rows, selected_entry, theme);
            Paragraph::new(footer_text(mode))
                .style(theme.footer)
                .render(body[1], frame.buffer_mut());
        }
        PickerMode::ConfirmDelete => {
            let body = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(chunks[1]);
            render_grouped_list(frame, body[0], rows, selected_entry, theme);
            render_grouped_confirmation_inline(
                frame,
                body[0],
                rows,
                selected_entry,
                entries[selected_entry].delete_prompt(),
                theme,
            );
        }
    }
}

fn render_grouped_frame(
    frame: &mut Frame,
    rows: &[GroupedPickerRow],
    selected_entry: usize,
    anchor_row: u16,
) {
    let area = panel_area(
        frame.area(),
        anchor_row,
        grouped_picker_content_lines(rows),
        PickerMode::Browse,
    );
    ClearWidget.render(area, frame.buffer_mut());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let theme = PickerTheme::omarchy();

    Paragraph::new("Select an entry:")
        .style(theme.title)
        .render(chunks[0], frame.buffer_mut());

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(chunks[1]);

    render_grouped_list(frame, body[0], rows, selected_entry, theme);
    Paragraph::new(footer_text(PickerMode::Browse))
        .style(theme.footer)
        .render(body[1], frame.buffer_mut());
}

fn footer_text(mode: PickerMode) -> &'static str {
    match mode {
        PickerMode::Browse => "j/Down next, k/Up previous, Enter confirm, q/Esc cancel",
        PickerMode::ConfirmDelete => "Delete selected item? [y/N]",
    }
}

fn panel_area(area: Rect, anchor_row: u16, entry_count: usize, mode: PickerMode) -> Rect {
    let desired_rows = match mode {
        PickerMode::Browse => entry_count.saturating_add(2),
        PickerMode::ConfirmDelete => entry_count.saturating_add(3),
    };
    let panel_height = area.height.min(desired_rows.max(3) as u16).max(1);
    let preferred_top = anchor_row
        .saturating_add(1)
        .min(area.height.saturating_sub(1));
    let panel_top = preferred_top.min(area.height.saturating_sub(panel_height));
    Rect::new(area.x, area.y + panel_top, area.width, panel_height)
}

fn confirm_delete_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('y'))
}

fn render_grouped_list(
    frame: &mut Frame,
    area: Rect,
    rows: &[GroupedPickerRow],
    selected_entry: usize,
    theme: PickerTheme,
) {
    let labels: Vec<String> = rows.iter().map(|row| row.label.clone()).collect();
    let selected_row = find_selected_row(rows, selected_entry);
    render_label_list(frame, area, &labels, selected_row, theme);
}

fn render_label_list(
    frame: &mut Frame,
    area: Rect,
    labels: &[String],
    selected: usize,
    theme: PickerTheme,
) {
    let items: Vec<_> = labels
        .iter()
        .map(|label| {
            let lines = label.lines().map(Line::from).collect::<Vec<_>>();
            ListItem::new(Text::from(lines)).style(theme.normal)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(theme.selected)
        .highlight_symbol("▌ ");

    let mut state = ListState::default().with_selected(Some(selected));
    StatefulWidget::render(list, area, frame.buffer_mut(), &mut state);

    if area.height == 0 {
        return;
    }

    let offset = state.offset();
    let relative = labels
        .iter()
        .skip(offset)
        .take(selected.saturating_sub(offset))
        .map(|label| label.lines().count() as u16)
        .sum::<u16>();
    if relative >= area.height {
        return;
    }

    let accent_area = Rect::new(area.x, area.y + relative, 2, 1);
    Paragraph::new(Span::styled("▌ ", theme.accent)).render(accent_area, frame.buffer_mut());
}

fn grouped_picker_content_lines(rows: &[GroupedPickerRow]) -> usize {
    rows.iter().map(|row| row.line_count).sum()
}

fn find_selected_row(rows: &[GroupedPickerRow], selected_entry: usize) -> usize {
    rows.iter()
        .position(|row| row.selectable_entry_index == Some(selected_entry))
        .unwrap_or(0)
}

fn build_grouped_rows<T: GroupedPickerItem>(entries: &[T]) -> Vec<GroupedPickerRow> {
    let mut rows = Vec::new();
    let mut current_date: Option<&str> = None;

    for (index, entry) in entries.iter().enumerate() {
        if current_date != Some(entry.group_date()) {
            if !rows.is_empty() {
                rows.push(GroupedPickerRow {
                    label: " ".to_string(),
                    selectable_entry_index: None,
                    line_count: 1,
                });
            }
            let heading = render_day_heading(entry.group_date());
            rows.push(GroupedPickerRow {
                label: format!("{heading}\n{}", render_day_separator(&heading)),
                selectable_entry_index: None,
                line_count: 2,
            });
            current_date = Some(entry.group_date());
        }

        rows.push(GroupedPickerRow {
            label: entry.grouped_display_label(),
            selectable_entry_index: Some(index),
            line_count: entry.grouped_line_count(),
        });
    }

    rows
}

fn build_grouped_done_rows(entries: &[LogEntry], states: &[EntryState]) -> Vec<GroupedPickerRow> {
    let mut rows = Vec::new();
    let mut current_date: Option<&str> = None;

    for (index, (entry, state)) in entries.iter().zip(states.iter()).enumerate() {
        if current_date != Some(entry.date.as_str()) {
            if !rows.is_empty() {
                rows.push(GroupedPickerRow {
                    label: " ".to_string(),
                    selectable_entry_index: None,
                    line_count: 1,
                });
            }
            let heading = render_day_heading(&entry.date);
            rows.push(GroupedPickerRow {
                label: format!("{heading}\n{}", render_day_separator(&heading)),
                selectable_entry_index: None,
                line_count: 2,
            });
            current_date = Some(entry.date.as_str());
        }

        let summary = render_entry_summary(&entry.summary, &entry.tags);
        let mut lines = vec![format!("{} {}  {}", state.display_marker(), summary, entry.time)];
        lines.extend(entry.notes.iter().map(|note| crate::log::model::format_note_display(note)));
        rows.push(GroupedPickerRow {
            label: lines.join("\n"),
            selectable_entry_index: Some(index),
            line_count: entry.display_line_count(),
        });
    }

    rows
}

fn render_grouped_confirmation_inline(
    frame: &mut Frame,
    area: Rect,
    rows: &[GroupedPickerRow],
    selected_entry: usize,
    prompt: &str,
    theme: PickerTheme,
) {
    if area.height == 0 {
        return;
    }

    let selected_row = find_selected_row(rows, selected_entry);
    let relative = rows
        .iter()
        .take(selected_row + 1)
        .map(|row| row.line_count as u16)
        .sum::<u16>();
    let confirm_y = area.y.saturating_add(relative);

    if confirm_y >= area.y.saturating_add(area.height) || area.width <= 2 {
        return;
    }

    let confirm_area = Rect::new(area.x + 2, confirm_y, area.width.saturating_sub(2), 1);
    Paragraph::new(prompt)
        .style(theme.footer)
        .render(confirm_area, frame.buffer_mut());
}

struct TerminalGuard {
    clear_area: Rect,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut stdout = io::stdout();
        for row in 0..self.clear_area.height {
            let _ = execute!(
                stdout,
                cursor::MoveTo(self.clear_area.x, self.clear_area.y + row),
                Clear(ClearType::CurrentLine)
            );
        }
        let _ = execute!(
            stdout,
            cursor::MoveTo(self.clear_area.x, self.clear_area.y),
            cursor::Show
        );
    }
}

#[allow(dead_code)]
fn _accept_modifier(_: KeyModifiers) {}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[derive(Debug, Clone)]
    struct TestSurface {
        rows: Vec<TestRow>,
    }

    #[derive(Debug, Clone)]
    struct TestRow {
        text: String,
        fg: Color,
        bg: Color,
    }

    fn render_test_surface<T: PickerItem + GroupedPickerItem>(
        entries: &[T],
        selected: usize,
        mode: PickerMode,
        _width: u16,
        height: u16,
    ) -> TestSurface {
        let theme = PickerTheme::omarchy();
        let rows_for_picker = build_grouped_rows(entries);
        let panel = panel_area(
            Rect::new(0, 0, 72, height),
            2,
            grouped_picker_content_lines(&rows_for_picker),
            mode,
        );
        let top_padding = panel.y as usize;
        let mut rows = Vec::with_capacity(top_padding + grouped_picker_content_lines(&rows_for_picker) + 3);
        rows.extend((0..top_padding).map(|_| TestRow {
            text: String::new(),
            fg: Color::Reset,
            bg: Color::Reset,
        }));
        rows.push(TestRow {
            text: "Select an entry:".to_string(),
            fg: theme.title.fg.unwrap_or(Color::Reset),
            bg: theme.title.bg.unwrap_or(Color::Reset),
        });
        let selected_row = find_selected_row(&rows_for_picker, selected);
        rows.extend(rows_for_picker.iter().enumerate().flat_map(|(index, row)| {
            let style = if row.selectable_entry_index == Some(selected) {
                theme.selected
            } else {
                theme.normal
            };
            row.label.lines().map(move |line| TestRow {
                text: if index == selected_row {
                    format!("▌ {line}")
                } else {
                    format!("  {line}")
                },
                fg: style.fg.unwrap_or(Color::Reset),
                bg: style.bg.unwrap_or(Color::Reset),
            })
        }));
        let footer = match mode {
            PickerMode::Browse => footer_text(mode).to_string(),
            PickerMode::ConfirmDelete => entries
                .get(selected)
                .map(|entry| entry.delete_prompt().to_string())
                .unwrap_or_else(|| footer_text(mode).to_string()),
        };
        rows.push(TestRow {
            text: footer,
            fg: theme.footer.fg.unwrap_or(Color::Reset),
            bg: theme.footer.bg.unwrap_or(Color::Reset),
        });
        TestSurface { rows }
    }

    fn surface_text(surface: &TestSurface) -> String {
        surface
            .rows
            .iter()
            .map(|row| row.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn find_cell<'a>(surface: &'a TestSurface, needle: &str) -> &'a TestRow {
        surface
            .rows
            .iter()
            .find(|row| row.text.contains(needle))
            .expect("row not found")
    }

    #[test]
    fn ratatui_render_shows_footer_help() {
        let entries = vec![crate::log::model::UnfinishedEntry {
            date: "2026-03-24".into(),
            time: "11:32".into(),
            summary: "spaced entry".into(),
            tags: Vec::new(),
            ordinal: 0,
            start: 0,
            end: 0,
        }];

        let rendered = render_test_surface(&entries, 0, PickerMode::Browse, 72, 8);

        assert!(surface_text(&rendered).contains("j/Down next"));
        assert!(surface_text(&rendered).contains("Enter confirm"));
    }

    #[test]
    fn ratatui_render_uses_multiline_item_height_for_generic_picker() {
        let entries = vec![crate::log::model::LogEntry {
            date: "2026-03-25".into(),
            time: "09:15".into(),
            summary: "selected".into(),
            tags: Vec::new(),
            notes: vec!["first note".into(), "second note".into()],
            state: crate::log::model::EntryState::Pending,
            ordinal: 0,
            start: 0,
            end: 0,
        }];

        let rendered = render_test_surface(&entries, 0, PickerMode::Browse, 72, 8);

        assert!(surface_text(&rendered).contains("first note"));
        assert!(surface_text(&rendered).contains("second note"));
        assert!(surface_text(&rendered).contains("Enter confirm"));
    }

    #[test]
    fn panel_area_is_anchored_to_bottom() {
        let area = panel_area(Rect::new(0, 0, 72, 20), 15, 3, PickerMode::Browse);

        assert_eq!(area.height, 5);
        assert_eq!(area.y, 15);
    }

    #[test]
    fn panel_area_prefers_to_render_below_cursor() {
        let area = panel_area(Rect::new(0, 0, 72, 20), 2, 3, PickerMode::Browse);

        assert_eq!(area.height, 5);
        assert_eq!(area.y, 3);
    }

    #[test]
    fn ratatui_render_highlights_selected_row() {
        let entries = vec![
            crate::log::model::UnfinishedEntry {
                date: "2026-03-24".into(),
                time: "11:32".into(),
                summary: "first".into(),
                tags: Vec::new(),
                ordinal: 0,
                start: 0,
                end: 0,
            },
            crate::log::model::UnfinishedEntry {
                date: "2026-03-25".into(),
                time: "09:15".into(),
                summary: "selected".into(),
                tags: Vec::new(),
                ordinal: 1,
                start: 0,
                end: 0,
            },
        ];

        let rendered = render_test_surface(&entries, 1, PickerMode::Browse, 72, 8);

        let selected = find_cell(&rendered, "selected  09:15");
        assert_eq!(selected.fg, Color::Black);
        assert!(selected.bg != Color::Reset);
    }

    #[test]
    fn ratatui_render_keeps_selection_visible_during_delete_confirmation() {
        let entries = vec![crate::log::model::LogEntry {
            date: "2026-03-25".into(),
            time: "09:15".into(),
            summary: "selected".into(),
            tags: Vec::new(),
            notes: Vec::new(),
            state: crate::log::model::EntryState::Pending,
            ordinal: 0,
            start: 0,
            end: 0,
        }];

        let rendered = render_test_surface(&entries, 0, PickerMode::ConfirmDelete, 72, 8);

        assert!(surface_text(&rendered).contains("Delete selected entry? [y/N]"));
        let confirm_index = rendered
            .rows
            .iter()
            .position(|row| row.text == "Delete selected entry? [y/N]")
            .unwrap();
        let selected_index = rendered
            .rows
            .iter()
            .position(|row| row.text.contains("□ selected  09:15"))
            .unwrap();
        assert_eq!(confirm_index, selected_index + 1);
        let selected = find_cell(&rendered, "□ selected  09:15");
        assert_eq!(selected.fg, Color::Black);
        assert!(selected.bg != Color::Reset);
    }

    #[test]
    fn ratatui_render_stays_in_bottom_panel() {
        let entries = vec![crate::log::model::UnfinishedEntry {
            date: "2026-03-24".into(),
            time: "11:32".into(),
            summary: "spaced entry".into(),
            tags: Vec::new(),
            ordinal: 0,
            start: 0,
            end: 0,
        }];

        let rendered = render_test_surface(&entries, 0, PickerMode::Browse, 72, 12);
        let title_index = rendered
            .rows
            .iter()
            .position(|row| row.text == "Select an entry:")
            .unwrap();

        assert!(title_index > 0);
        assert!(
            rendered.rows[..title_index]
                .iter()
                .all(|row| row.text.is_empty())
        );
    }

    #[test]
    fn grouped_rows_use_visible_spacer_between_dates() {
        let entries = vec![
            crate::log::model::LogEntry {
                date: "2026-03-25".into(),
                time: "08:01".into(),
                summary: "older".into(),
                tags: Vec::new(),
                notes: vec!["older note".into()],
                state: crate::log::model::EntryState::Pending,
                ordinal: 0,
                start: 0,
                end: 0,
            },
            crate::log::model::LogEntry {
                date: "2026-03-26".into(),
                time: "09:12".into(),
                summary: "newer".into(),
                tags: Vec::new(),
                notes: vec!["newer note".into()],
                state: crate::log::model::EntryState::Pending,
                ordinal: 1,
                start: 0,
                end: 0,
            },
        ];

        let rows = build_grouped_rows(&entries);

        assert_eq!(rows[2].label, " ");
        assert_eq!(rows[2].line_count, 1);
        assert_eq!(rows[2].selectable_entry_index, None);
    }
}
