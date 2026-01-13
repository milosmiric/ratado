//! Sidebar widget with projects and tags.
//!
//! Displays the project list and tag list in a vertical split.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, FocusPanel};

/// Renders the sidebar with projects and tags.
pub fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::Sidebar;

    // Split into projects and tags sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // Projects
            Constraint::Length(1),  // Separator
            Constraint::Min(5),     // Tags
        ])
        .split(area);

    render_projects(frame, app, chunks[0], is_focused);
    render_separator(frame, chunks[1]);
    render_tags(frame, app, chunks[2]);
}

/// Renders the projects list.
fn render_projects(frame: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            " PROJECTS ",
            Style::default()
                .fg(if is_focused { Color::Cyan } else { Color::White })
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::RIGHT)
        .border_style(border_style);

    // Build project list items
    let mut items: Vec<ListItem> = Vec::new();

    // "All Tasks" option
    let all_selected = app.selected_project_index == 0;
    let all_count = app.total_task_count();
    items.push(create_project_item("All Tasks", all_count, all_selected, is_focused, None));

    // Project items
    for (i, project) in app.projects.iter().enumerate() {
        let selected = app.selected_project_index == i + 1;
        let count = app.task_count_for_project(&project.id);
        items.push(create_project_item(
            &project.name,
            count,
            selected,
            is_focused,
            Some(&project.icon),
        ));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Creates a styled list item for a project.
fn create_project_item<'a>(
    name: &str,
    count: usize,
    selected: bool,
    focused: bool,
    icon: Option<&str>,
) -> ListItem<'a> {
    let prefix = if selected { "›" } else { " " };
    let icon_str = icon.unwrap_or("📋");

    let style = if selected && focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if selected {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    let content = format!("{} {} {} ({})", prefix, icon_str, name, count);
    ListItem::new(Line::from(Span::styled(content, style)))
}

/// Renders a separator line.
fn render_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new("─".repeat(area.width as usize))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(sep, area);
}

/// Renders the tags list.
fn render_tags(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " TAGS ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    if app.tags.is_empty() {
        let empty = Paragraph::new("  No tags yet")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .tags
        .iter()
        .map(|tag| {
            let count = app.task_count_for_tag(&tag.name);
            let content = format!("  # {} ({})", tag.name, count);
            ListItem::new(Line::from(Span::styled(
                content,
                Style::default().fg(Color::Gray),
            )))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
