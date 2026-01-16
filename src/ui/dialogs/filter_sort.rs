//! Filter and sort selection dialog.
//!
//! A popup dialog for selecting filter and sort options with keyboard navigation.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use super::{centered_rect, dialog_block, field_block, hint_style, selected_style, DialogAction};
use crate::models::{Filter, SortOrder, Task};
use crate::ui::theme;

/// Which section of the dialog is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterSortSection {
    #[default]
    Filter,
    Sort,
}

/// Dialog for selecting filter and sort options.
#[derive(Debug, Clone)]
pub struct FilterSortDialog {
    /// Currently focused section
    pub section: FilterSortSection,
    /// Selected filter index
    pub filter_index: usize,
    /// Selected sort index
    pub sort_index: usize,
    /// Task counts for each filter option
    pub filter_counts: Vec<usize>,
}

impl FilterSortDialog {
    /// All available filter options.
    const FILTERS: &'static [(Filter, &'static str, &'static str)] = &[
        (Filter::All, "All", "Show all tasks"),
        (Filter::Pending, "Pending", "Tasks not yet completed"),
        (Filter::Completed, "Completed", "Finished tasks"),
        (Filter::DueToday, "Due Today", "Tasks due today"),
        (Filter::DueThisWeek, "Due This Week", "Tasks due within 7 days"),
        (Filter::Overdue, "Overdue", "Past due tasks"),
        (Filter::ByPriority(crate::models::Priority::Urgent), "Urgent", "Urgent priority only"),
        (Filter::ByPriority(crate::models::Priority::High), "High", "High priority only"),
        (Filter::ByPriority(crate::models::Priority::Medium), "Medium", "Medium priority only"),
        (Filter::ByPriority(crate::models::Priority::Low), "Low", "Low priority only"),
    ];

    /// All available sort options.
    const SORTS: &'static [(SortOrder, &'static str, &'static str)] = &[
        (SortOrder::DueDateAsc, "Due Date", "Earliest due first"),
        (SortOrder::PriorityDesc, "Priority", "Highest priority first"),
        (SortOrder::CreatedDesc, "Created", "Newest first"),
        (SortOrder::Alphabetical, "Alphabetical", "A-Z by title"),
    ];

    /// Creates a new dialog with current filter/sort pre-selected.
    pub fn new(current_filter: &Filter, current_sort: &SortOrder, tasks: &[Task]) -> Self {
        let filter_index = Self::FILTERS
            .iter()
            .position(|(f, _, _)| std::mem::discriminant(f) == std::mem::discriminant(current_filter))
            .unwrap_or(0);

        let sort_index = Self::SORTS
            .iter()
            .position(|(s, _, _)| s == current_sort)
            .unwrap_or(0);

        // Calculate counts for each filter
        let filter_counts = Self::FILTERS
            .iter()
            .map(|(filter, _, _)| Self::count_matching_tasks(filter, tasks))
            .collect();

        Self {
            section: FilterSortSection::Filter,
            filter_index,
            sort_index,
            filter_counts,
        }
    }

    /// Counts how many tasks match a given filter.
    fn count_matching_tasks(filter: &Filter, tasks: &[Task]) -> usize {
        tasks.iter().filter(|t| filter.matches(t)).count()
    }

    /// Returns the currently selected filter.
    pub fn selected_filter(&self) -> Filter {
        Self::FILTERS[self.filter_index].0.clone()
    }

    /// Returns the currently selected sort order.
    pub fn selected_sort(&self) -> SortOrder {
        Self::SORTS[self.sort_index].0
    }

    /// Handles a key event and returns the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            // Cancel
            KeyCode::Esc | KeyCode::Char('q') => DialogAction::Cancel,

            // Confirm selection
            KeyCode::Enter => DialogAction::Submit,

            // Switch between filter and sort sections
            KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                self.section = match self.section {
                    FilterSortSection::Filter => FilterSortSection::Sort,
                    FilterSortSection::Sort => FilterSortSection::Filter,
                };
                DialogAction::None
            }

            // Navigate within section
            KeyCode::Up | KeyCode::Char('k') => {
                match self.section {
                    FilterSortSection::Filter => {
                        self.filter_index = self.filter_index.saturating_sub(1);
                    }
                    FilterSortSection::Sort => {
                        self.sort_index = self.sort_index.saturating_sub(1);
                    }
                }
                DialogAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.section {
                    FilterSortSection::Filter => {
                        self.filter_index = (self.filter_index + 1).min(Self::FILTERS.len() - 1);
                    }
                    FilterSortSection::Sort => {
                        self.sort_index = (self.sort_index + 1).min(Self::SORTS.len() - 1);
                    }
                }
                DialogAction::None
            }

            // Jump to top/bottom
            KeyCode::Home | KeyCode::Char('g') => {
                match self.section {
                    FilterSortSection::Filter => self.filter_index = 0,
                    FilterSortSection::Sort => self.sort_index = 0,
                }
                DialogAction::None
            }
            KeyCode::End | KeyCode::Char('G') => {
                match self.section {
                    FilterSortSection::Filter => self.filter_index = Self::FILTERS.len() - 1,
                    FilterSortSection::Sort => self.sort_index = Self::SORTS.len() - 1,
                }
                DialogAction::None
            }

            _ => DialogAction::None,
        }
    }

    /// Renders the dialog to the frame.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dialog dimensions
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 18.min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Render dimmed background
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(theme::BG_DARK)),
            area,
        );

        // Render dialog box with themed styling
        let block = dialog_block("Filter & Sort", false);
        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split into two columns
        let columns = Layout::horizontal([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(inner);

        // Render filter column
        self.render_filter_column(frame, columns[0]);

        // Render sort column
        self.render_sort_column(frame, columns[1]);
    }

    fn render_filter_column(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.section == FilterSortSection::Filter;
        let block = field_block("Filter", is_focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        for (i, (_, name, desc)) in Self::FILTERS.iter().enumerate() {
            let is_selected = i == self.filter_index;
            let count = self.filter_counts.get(i).copied().unwrap_or(0);
            let style = if is_selected && is_focused {
                selected_style()
            } else if is_selected {
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };

            let count_style = if is_selected && is_focused {
                selected_style()
            } else {
                hint_style()
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(*name, style),
                Span::styled(format!(" ({})", count), count_style),
            ]));

            // Show description for selected item
            if is_selected {
                lines.push(Line::from(Span::styled(
                    format!("    {}", desc),
                    hint_style(),
                )));
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn render_sort_column(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.section == FilterSortSection::Sort;
        let block = field_block("Sort", is_focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        for (i, (_, name, desc)) in Self::SORTS.iter().enumerate() {
            let is_selected = i == self.sort_index;
            let style = if is_selected && is_focused {
                selected_style()
            } else if is_selected {
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(*name, style),
            ]));

            // Show description for selected item
            if is_selected {
                lines.push(Line::from(Span::styled(
                    format!("    {}", desc),
                    hint_style(),
                )));
            }
        }

        // Add help text at bottom
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Tab:switch  ↑↓:select  Enter:apply",
            hint_style(),
        )));

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}

impl Default for FilterSortDialog {
    fn default() -> Self {
        Self::new(&Filter::All, &SortOrder::DueDateAsc, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_new_dialog() {
        let dialog = FilterSortDialog::new(&Filter::Pending, &SortOrder::PriorityDesc, &[]);
        assert_eq!(dialog.filter_index, 1); // Pending is index 1
        assert_eq!(dialog.sort_index, 1); // PriorityDesc is index 1
    }

    #[test]
    fn test_navigation_down() {
        let mut dialog = FilterSortDialog::default();
        assert_eq!(dialog.filter_index, 0);
        dialog.handle_key(key(KeyCode::Down));
        assert_eq!(dialog.filter_index, 1);
    }

    #[test]
    fn test_navigation_up() {
        let mut dialog = FilterSortDialog::new(&Filter::Completed, &SortOrder::DueDateAsc, &[]);
        assert_eq!(dialog.filter_index, 2);
        dialog.handle_key(key(KeyCode::Up));
        assert_eq!(dialog.filter_index, 1);
    }

    #[test]
    fn test_section_switch() {
        let mut dialog = FilterSortDialog::default();
        assert_eq!(dialog.section, FilterSortSection::Filter);
        dialog.handle_key(key(KeyCode::Tab));
        assert_eq!(dialog.section, FilterSortSection::Sort);
        dialog.handle_key(key(KeyCode::Tab));
        assert_eq!(dialog.section, FilterSortSection::Filter);
    }

    #[test]
    fn test_escape_cancels() {
        let mut dialog = FilterSortDialog::default();
        assert_eq!(dialog.handle_key(key(KeyCode::Esc)), DialogAction::Cancel);
    }

    #[test]
    fn test_enter_submits() {
        let mut dialog = FilterSortDialog::default();
        assert_eq!(dialog.handle_key(key(KeyCode::Enter)), DialogAction::Submit);
    }
}
