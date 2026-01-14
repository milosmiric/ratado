//! Search view for finding tasks.
//!
//! Provides full-text search functionality with live filtering and result highlighting.

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{Priority, Task, TaskStatus};
use crate::utils::format_relative_date;

/// A search result with match information.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching task
    pub task: Task,
    /// Match position in title (start, end)
    pub title_match: Option<(usize, usize)>,
    /// Snippet from description with match
    pub desc_snippet: Option<String>,
}

/// Performs search on tasks and returns matching results.
///
/// Searches in both task title and description (case-insensitive).
pub fn search_tasks(query: &str, tasks: &[Task]) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    if query_lower.is_empty() {
        return Vec::new();
    }

    tasks
        .iter()
        .filter_map(|task| {
            let title_lower = task.title.to_lowercase();
            let title_match = title_lower.find(&query_lower).map(|start| {
                (start, start + query_lower.len())
            });

            let desc_snippet = task.description.as_ref().and_then(|desc| {
                let desc_lower = desc.to_lowercase();
                desc_lower.find(&query_lower).map(|pos| {
                    // Extract snippet around match
                    let start = pos.saturating_sub(20);
                    let end = (pos + query_lower.len() + 30).min(desc.len());
                    let prefix = if start > 0 { "..." } else { "" };
                    let suffix = if end < desc.len() { "..." } else { "" };
                    format!("{}{}{}", prefix, &desc[start..end], suffix)
                })
            });

            if title_match.is_some() || desc_snippet.is_some() {
                Some(SearchResult {
                    task: task.clone(),
                    title_match,
                    desc_snippet,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Renders the search view.
pub fn render_search(
    frame: &mut Frame,
    query: &str,
    cursor_pos: usize,
    results: &[SearchResult],
    selected_index: usize,
    area: Rect,
) {
    render_search_with_context(frame, query, cursor_pos, results, selected_index, area, None);
}

/// Renders the search view with optional project context.
pub fn render_search_with_context(
    frame: &mut Frame,
    query: &str,
    cursor_pos: usize,
    results: &[SearchResult],
    selected_index: usize,
    area: Rect,
    project_name: Option<&str>,
) {
    // Clear the area
    frame.render_widget(Clear, area);

    // Main layout: search input at top, results below
    let chunks = Layout::vertical([
        Constraint::Length(3), // Search input
        Constraint::Min(0),    // Results
    ])
    .split(area);

    // Render search input with project context
    render_search_input_with_context(frame, query, cursor_pos, chunks[0], project_name);

    // Render results
    render_search_results(frame, query, results, selected_index, chunks[1]);
}

/// Renders the search input box with optional project context.
fn render_search_input_with_context(
    frame: &mut Frame,
    query: &str,
    cursor_pos: usize,
    area: Rect,
    project_name: Option<&str>,
) {
    let title = match project_name {
        Some(name) if name != "All Tasks" => format!(" Search in: {} ", name),
        _ => " Search Tasks ".to_string(),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show search query with cursor
    let display_text = format!("/{}", query);
    let mut spans = Vec::new();

    for (i, c) in display_text.chars().enumerate() {
        let style = if i == cursor_pos + 1 {
            // Cursor position (offset by 1 for the '/')
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(c.to_string(), style));
    }

    // Show cursor at end if at end of text
    if cursor_pos >= query.len() {
        spans.push(Span::styled(" ", Style::default().bg(Color::Yellow)));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, inner);
}

/// Renders the search results list.
fn render_search_results(
    frame: &mut Frame,
    query: &str,
    results: &[SearchResult],
    selected_index: usize,
    area: Rect,
) {
    let block = Block::default()
        .title(format!(" Results ({}) ", results.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if results.is_empty() {
        let msg = if query.is_empty() {
            "Type to search..."
        } else {
            "No matching tasks found"
        };
        let paragraph = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    let visible_height = inner.height as usize;

    // Calculate scroll offset to keep selected item visible
    // Each result takes 2 lines (task + description/spacing)
    let lines_per_result = 2;
    let visible_results = visible_height / lines_per_result;
    let scroll_offset = if selected_index >= visible_results {
        selected_index - visible_results + 1
    } else {
        0
    };

    for (i, result) in results.iter().enumerate().skip(scroll_offset) {
        if lines.len() >= visible_height {
            break;
        }

        let is_selected = i == selected_index;

        // Render task row similar to task list
        let task_line = render_task_result(&result.task, result.title_match, is_selected, inner.width);
        lines.push(task_line);

        // Render description snippet if present (and we have room)
        if lines.len() < visible_height {
            if let Some(ref snippet) = result.desc_snippet {
                let desc_line = render_description_snippet(snippet, query, is_selected);
                lines.push(desc_line);
            } else {
                // Empty line for spacing
                lines.push(Line::from(""));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Renders a task result row similar to the main task list.
fn render_task_result(
    task: &Task,
    title_match: Option<(usize, usize)>,
    is_selected: bool,
    width: u16,
) -> Line<'static> {
    // Checkbox based on status
    let checkbox = match task.status {
        TaskStatus::Pending => "[ ]",
        TaskStatus::InProgress => "[▸]",
        TaskStatus::Completed | TaskStatus::Archived => "[✓]",
    };

    // Priority indicator
    let priority = match task.priority {
        Priority::Urgent => "!!",
        Priority::High => " !",
        Priority::Medium => "  ",
        Priority::Low => " ↓",
    };

    // Priority color
    let priority_style = match task.priority {
        Priority::Urgent => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Priority::High => Style::default().fg(Color::Yellow),
        Priority::Medium => Style::default(),
        Priority::Low => Style::default().fg(Color::DarkGray),
    };

    // Due date
    let due_str = task
        .due_date
        .map(format_relative_date)
        .unwrap_or_default();

    // Calculate available width for title
    let fixed_width = 5 + 3 + 3 + 2 + due_str.len() + 2; // selector + checkbox + priority + spacing + due
    let title_width = (width as usize).saturating_sub(fixed_width).max(10);

    // Base style based on task state
    let base_style = if task.status == TaskStatus::Completed || task.status == TaskStatus::Archived {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    } else if task.is_overdue() {
        Style::default().fg(Color::Red)
    } else if task.is_due_today() {
        Style::default().fg(Color::Yellow)
    } else if task.is_due_this_week() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    // Selection indicator and style
    let (selector, selector_style) = if is_selected {
        ("▶ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        ("  ", Style::default())
    };

    // Row style with selection highlight
    let row_style = if is_selected {
        base_style.add_modifier(Modifier::BOLD)
    } else {
        base_style
    };

    // Build spans
    let mut spans = vec![
        Span::styled(selector.to_string(), selector_style),
        Span::styled(format!("{} ", checkbox), row_style),
        Span::styled(format!("{} ", priority), priority_style),
    ];

    // Add title with match highlighting
    let title = &task.title;
    if let Some((start, end)) = title_match {
        // Before match
        if start > 0 {
            let before = truncate_str(&title[..start], title_width);
            spans.push(Span::styled(before, row_style));
        }

        // The match (highlighted with underline)
        let match_text = &title[start..end.min(title.len())];
        let match_style = row_style.add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
        spans.push(Span::styled(match_text.to_string(), match_style));

        // After match
        if end < title.len() {
            let remaining_width = title_width.saturating_sub(end);
            let after = truncate_str(&title[end..], remaining_width);
            spans.push(Span::styled(after, row_style));
        }
    } else {
        // No match in title
        let truncated = truncate_str(title, title_width);
        spans.push(Span::styled(truncated, row_style));
    }

    // Add due date
    if !due_str.is_empty() {
        spans.push(Span::styled(
            format!("  {}", due_str),
            Style::default().fg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

/// Renders a description snippet with the match highlighted.
fn render_description_snippet(snippet: &str, query: &str, _is_selected: bool) -> Line<'static> {
    let indent = "     "; // Align with task title after selector + checkbox + priority
    let base_style = Style::default().fg(Color::DarkGray);

    // Find and highlight the match in the snippet
    let query_lower = query.to_lowercase();
    let snippet_lower = snippet.to_lowercase();

    if let Some(pos) = snippet_lower.find(&query_lower) {
        let before = &snippet[..pos];
        let matched = &snippet[pos..pos + query.len()];
        let after = &snippet[pos + query.len()..];

        let match_style = base_style.add_modifier(Modifier::UNDERLINED | Modifier::BOLD);

        Line::from(vec![
            Span::styled(indent.to_string(), Style::default()),
            Span::styled(before.to_string(), base_style),
            Span::styled(matched.to_string(), match_style),
            Span::styled(after.to_string(), base_style),
        ])
    } else {
        Line::from(vec![
            Span::styled(indent.to_string(), Style::default()),
            Span::styled(snippet.to_string(), base_style),
        ])
    }
}

/// Truncates a string to fit within a given width, adding ellipsis if needed.
fn truncate_str(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 3 {
        format!("{}...", &s[..max_width - 3])
    } else {
        s[..max_width].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;

    fn sample_tasks() -> Vec<Task> {
        vec![
            {
                let mut t = Task::new("Buy groceries");
                t.description = Some("Get milk and bread from the store".to_string());
                t
            },
            {
                let mut t = Task::new("Write documentation");
                t.description = Some("Update the README file".to_string());
                t
            },
            {
                let mut t = Task::new("Fix bug in search");
                t.priority = Priority::High;
                t
            },
        ]
    }

    #[test]
    fn test_search_by_title() {
        let tasks = sample_tasks();
        let results = search_tasks("groceries", &tasks);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task.title, "Buy groceries");
        assert!(results[0].title_match.is_some());
    }

    #[test]
    fn test_search_by_description() {
        let tasks = sample_tasks();
        let results = search_tasks("README", &tasks);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task.title, "Write documentation");
        assert!(results[0].desc_snippet.is_some());
    }

    #[test]
    fn test_search_case_insensitive() {
        let tasks = sample_tasks();
        let results = search_tasks("BUG", &tasks);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task.title, "Fix bug in search");
    }

    #[test]
    fn test_search_no_results() {
        let tasks = sample_tasks();
        let results = search_tasks("nonexistent", &tasks);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let tasks = sample_tasks();
        let results = search_tasks("", &tasks);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_multiple_results() {
        let tasks = sample_tasks();
        // "the" appears in two tasks
        let results = search_tasks("the", &tasks);
        assert_eq!(results.len(), 2);
    }
}
