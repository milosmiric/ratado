//! Sidebar widget with projects list.
//!
//! Displays the project list for filtering tasks with a modern, clean design.
//! When focused, users can navigate projects using j/k or arrow keys.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, FocusPanel};
use super::theme::{self, icons};

/// Renders the sidebar with projects.
pub fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::Sidebar;
    render_projects(frame, app, area, is_focused);
}

/// Renders the projects list with modern styling.
fn render_projects(frame: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    // Title style indicates focus - border stays consistent for visibility
    let title_style = if is_focused {
        Style::default()
            .fg(theme::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme::TEXT_MUTED)
    };

    let block = Block::default()
        .title(Span::styled(" Projects ", title_style))
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme::BORDER));

    // Build project list items
    let mut items: Vec<ListItem> = Vec::new();

    // "All Tasks" option
    let all_selected = app.selected_project_index == 0;
    let all_count = app.total_task_count();
    items.push(create_project_item(
        "All Tasks",
        all_count,
        all_selected,
        is_focused,
        None,
        None,
    ));

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
            Some(&project.color),
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
    color: Option<&str>,
) -> ListItem<'a> {
    let icon_str = icon.unwrap_or("📋");

    // Determine the base style based on selection and focus state
    let (text_style, count_style, bg_style) = if selected && focused {
        (
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(theme::PRIMARY_LIGHT),
            Some(theme::BG_SELECTION),
        )
    } else if selected {
        (
            Style::default().fg(theme::TEXT_PRIMARY),
            Style::default().fg(theme::TEXT_MUTED),
            Some(theme::BG_ELEVATED),
        )
    } else {
        (
            Style::default().fg(theme::TEXT_SECONDARY),
            Style::default().fg(theme::TEXT_MUTED),
            None,
        )
    };

    // Selection indicator
    let selector = if selected && focused {
        Span::styled(
            format!("{} ", icons::SELECTOR),
            Style::default().fg(theme::PRIMARY_LIGHT),
        )
    } else {
        Span::raw("  ")
    };

    // Color indicator for projects
    let color_indicator = if let Some(hex) = color {
        let rgb = parse_hex_color(hex);
        Span::styled(format!("{} ", icons::CIRCLE), Style::default().fg(rgb))
    } else if icon.is_none() {
        // "All Tasks" - use a special icon
        Span::styled(
            format!("{} ", icons::SPARKLE),
            Style::default().fg(theme::SECONDARY),
        )
    } else {
        Span::raw("")
    };

    // Build the line
    let spans = vec![
        selector,
        color_indicator,
        Span::styled(format!("{} {}", icon_str, name), text_style),
        Span::styled(format!(" {}", count), count_style),
    ];

    // Create the list item with optional background
    let line = Line::from(spans);
    let mut item = ListItem::new(line);

    if let Some(bg) = bg_style {
        item = item.style(Style::default().bg(bg));
    }

    item
}

/// Parses a hex color string (e.g., "#3498db") to a Color.
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Rgb(128, 128, 128);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);

    Color::Rgb(r, g, b)
}
