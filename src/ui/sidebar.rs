//! Sidebar widget with projects list.
//!
//! Displays the project list for filtering tasks.
//! When focused, users can navigate projects using j/k or arrow keys.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, FocusPanel};

/// Renders the sidebar with projects.
pub fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusPanel::Sidebar;
    render_projects(frame, app, area, is_focused);
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
    items.push(create_project_item("All Tasks", all_count, all_selected, is_focused, None, None));

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

    // Add color indicator if project has a color
    let color_indicator = if let Some(hex) = color {
        let rgb = parse_hex_color(hex);
        Span::styled("● ", Style::default().fg(rgb))
    } else {
        Span::raw("  ")
    };

    let spans = vec![
        Span::styled(prefix, style),
        color_indicator,
        Span::styled(format!("{} {} ({})", icon_str, name, count), style),
    ];

    ListItem::new(Line::from(spans))
}

/// Parses a hex color string (e.g., "#3498db") to a Color.
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Gray;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);

    Color::Rgb(r, g, b)
}
