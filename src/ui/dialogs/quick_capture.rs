//! Quick Capture dialog for rapid task entry.
//!
//! Provides a spotlight-style overlay that lets users create tasks from a single
//! line using inline syntax: `<title> [@project] [#tag ...] [!priority] [due:date]`.
//!
//! ## Syntax
//!
//! | Token | Meaning | Examples |
//! |-------|---------|---------|
//! | `@Name` | Project (fuzzy matched) | `@Work`, `@back` → "Backend" |
//! | `#tag` | Tag | `#urgent`, `#bug` |
//! | `!1`–`!4` | Priority (1=urgent, 4=low) | `!1` |
//! | `due:val` | Due date | `due:tomorrow`, `due:fri` |
//! | `\@` `\#` | Escaped literals | `\@email` stays in title |
//! | Everything else | Title | Joined remaining words |

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::add_task::parse_due_date;
use super::AddTaskDialog;
use crate::models::{Priority, Project, Task};
use crate::storage::Tag;
use crate::ui::input::TextInput;
use crate::ui::theme;

// ─────────────────────────────────────────────────────────────────────────────
// Suggestion types
// ─────────────────────────────────────────────────────────────────────────────

/// What kind of autocomplete suggestions are currently showing.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SuggestionMode {
    /// No suggestions visible
    None,
    /// Showing project suggestions (triggered by `@`)
    Projects,
    /// Showing tag suggestions (triggered by `#`)
    Tags,
    /// Showing priority suggestions (triggered by `!`)
    Priorities,
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

/// Parsed result from quick capture input.
///
/// Contains the extracted title, project, tags, priority, and due date
/// from a single-line capture string.
#[derive(Debug, Clone, Default)]
pub struct ParsedCapture {
    /// The task title (remaining words after token extraction)
    pub title: String,
    /// Project name from `@Name` token
    pub project_name: Option<String>,
    /// Tags from `#tag` tokens
    pub tags: Vec<String>,
    /// Priority from `!1`–`!4` token
    pub priority: Option<Priority>,
    /// Raw due date text from `due:value` token
    pub due_date_text: Option<String>,
    /// Parsed due date
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Parses a quick capture input string into structured components.
///
/// Walks each whitespace-delimited word and classifies it:
/// - `@word` → project name
/// - `#word` → tag
/// - `!1`–`!4` → priority (1=urgent, 4=low)
/// - `due:rest` → due date text, parsed via `parse_due_date()`
/// - `\@word` / `\#word` → unescaped into title
/// - Everything else → title word
///
/// Tokens are only recognized after whitespace or at position 0.
///
/// # Arguments
///
/// * `input` - The raw capture input string
///
/// # Returns
///
/// A [`ParsedCapture`] with extracted fields
pub fn parse_capture_input(input: &str) -> ParsedCapture {
    let mut result = ParsedCapture::default();
    let mut title_words: Vec<String> = Vec::new();

    for word in input.split_whitespace() {
        if let Some(name) = word.strip_prefix('@') {
            if !name.is_empty() {
                result.project_name = Some(name.to_string());
            } else {
                title_words.push(word.to_string());
            }
        } else if let Some(tag) = word.strip_prefix('#') {
            if !tag.is_empty() {
                result.tags.push(tag.to_string());
            } else {
                title_words.push(word.to_string());
            }
        } else if let Some(level) = word.strip_prefix('!') {
            match level {
                "1" => result.priority = Some(Priority::Urgent),
                "2" => result.priority = Some(Priority::High),
                "3" => result.priority = Some(Priority::Medium),
                "4" => result.priority = Some(Priority::Low),
                _ => title_words.push(word.to_string()),
            }
        } else if let Some(date_str) = word.strip_prefix("due:") {
            if !date_str.is_empty() {
                result.due_date_text = Some(date_str.to_string());
                result.due_date = parse_due_date(date_str);
            }
        } else if let Some(rest) = word.strip_prefix("\\@") {
            title_words.push(format!("@{}", rest));
        } else if let Some(rest) = word.strip_prefix("\\#") {
            title_words.push(format!("#{}", rest));
        } else {
            title_words.push(word.to_string());
        }
    }

    result.title = title_words.join(" ");
    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Dialog
// ─────────────────────────────────────────────────────────────────────────────

/// Actions that can result from quick capture interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickCaptureAction {
    /// No action needed, dialog continues
    None,
    /// User submitted the task (Enter)
    Submit,
    /// User cancelled (Esc)
    Cancel,
    /// User wants to expand to full AddTaskDialog (Tab)
    ExpandToFull,
}

/// Quick Capture dialog for rapid task entry.
///
/// Provides a spotlight-style overlay where users type a single line
/// with inline syntax tokens that are parsed in real-time. Supports
/// autocomplete dropdowns for projects (`@`) and tags (`#`).
#[derive(Debug, Clone)]
pub struct QuickCaptureDialog {
    /// Text input field
    pub input: TextInput,
    /// Parsed result from current input
    pub parsed: ParsedCapture,
    /// Available projects for fuzzy matching
    pub projects: Vec<Project>,
    /// Available tags for reference
    pub all_tags: Vec<Tag>,
    /// Project matched from `@name` token
    pub matched_project: Option<Project>,
    /// Suggestion dropdown items
    suggestions: Vec<String>,
    /// Currently highlighted suggestion index
    selected_suggestion: Option<usize>,
    /// What kind of suggestions are showing
    suggestion_mode: SuggestionMode,
    /// Explicitly selected project (from dropdown, survives reparse)
    explicit_project: Option<Project>,
    /// Map of project_id → tag names used in that project's tasks
    project_tag_map: HashMap<String, Vec<String>>,
}

impl QuickCaptureDialog {
    /// Creates a new Quick Capture dialog.
    ///
    /// # Arguments
    ///
    /// * `projects` - Available projects for `@name` fuzzy matching
    /// * `tags` - Available tags for reference
    /// * `tasks` - Current tasks, used to build project-tag associations
    pub fn new(projects: Vec<Project>, tags: Vec<Tag>, tasks: &[Task]) -> Self {
        // Precompute which tags are used in each project's tasks
        let mut project_tag_map: HashMap<String, Vec<String>> = HashMap::new();
        for task in tasks {
            if let Some(ref pid) = task.project_id {
                let entry = project_tag_map.entry(pid.clone()).or_default();
                for tag in &task.tags {
                    if !entry.contains(tag) {
                        entry.push(tag.clone());
                    }
                }
            }
        }

        Self {
            input: TextInput::new().with_placeholder("Task title @project #tag !priority due:date"),
            parsed: ParsedCapture::default(),
            projects,
            all_tags: tags,
            matched_project: None,
            suggestions: Vec::new(),
            selected_suggestion: None,
            suggestion_mode: SuggestionMode::None,
            explicit_project: None,
            project_tag_map,
        }
    }

    /// Sets the pre-selected project (e.g. from the currently active sidebar selection).
    pub fn set_project(&mut self, project: Project) {
        self.explicit_project = Some(project);
    }

    /// Returns the explicitly selected project, if any.
    pub fn project(&self) -> Option<&Project> {
        self.explicit_project.as_ref()
    }

    /// Handles a key event and returns the resulting action.
    ///
    /// When suggestions are visible, Tab accepts the selected suggestion,
    /// and Up/Down navigate the dropdown. Otherwise Tab expands to full dialog.
    pub fn handle_key(&mut self, key: KeyEvent) -> QuickCaptureAction {
        let has_suggestions = !self.suggestions.is_empty();

        match key.code {
            KeyCode::Enter => {
                self.reparse();
                QuickCaptureAction::Submit
            }
            KeyCode::Esc => QuickCaptureAction::Cancel,
            KeyCode::Tab => {
                if has_suggestions {
                    self.accept_suggestion();
                    QuickCaptureAction::None
                } else {
                    self.reparse();
                    QuickCaptureAction::ExpandToFull
                }
            }
            KeyCode::Up if has_suggestions => {
                if let Some(idx) = self.selected_suggestion {
                    self.selected_suggestion = if idx == 0 {
                        Some(self.suggestions.len() - 1)
                    } else {
                        Some(idx - 1)
                    };
                }
                QuickCaptureAction::None
            }
            KeyCode::Down if has_suggestions => {
                if let Some(idx) = self.selected_suggestion {
                    self.selected_suggestion = Some((idx + 1) % self.suggestions.len());
                }
                QuickCaptureAction::None
            }
            // Word navigation
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                self.input.move_word_left();
                self.reparse();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                self.input.move_word_right();
                self.reparse();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
                self.input.delete_word_backward();
                self.reparse();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Char(c) => {
                self.input.insert(c);
                self.reparse();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Backspace => {
                self.input.delete_backward();
                self.reparse();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Delete => {
                self.input.delete_forward();
                self.reparse();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                self.input.move_word_left();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                self.input.move_word_right();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Left => {
                self.input.move_left();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Right => {
                self.input.move_right();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::Home => {
                self.input.move_home();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            KeyCode::End => {
                self.input.move_end();
                self.update_suggestions();
                QuickCaptureAction::None
            }
            _ => QuickCaptureAction::None,
        }
    }

    /// Re-parses the current input and updates parsed state and matched project.
    ///
    /// If `explicit_project` is set and no new `@token` appears in input,
    /// keeps the explicit project. If a new `@token` appears, clears it
    /// (user wants to change project).
    fn reparse(&mut self) {
        self.parsed = parse_capture_input(self.input.value());
        self.matched_project = self
            .parsed
            .project_name
            .as_ref()
            .and_then(|name| self.fuzzy_match_project(name));

        // If user typed a new @token, clear the explicit project
        if self.parsed.project_name.is_some() && self.explicit_project.is_some() {
            self.explicit_project = None;
        }
    }

    /// Finds the active `@partial` or `#partial` token at the cursor position.
    ///
    /// Scans backwards from the cursor to find if the cursor is currently
    /// inside a `@` or `#` token. Returns the mode and the partial text
    /// (without the prefix character), along with the byte offsets of the
    /// full token (including prefix) for replacement.
    ///
    /// # Returns
    ///
    /// `Some((mode, partial, start, end))` if cursor is in a token, `None` otherwise.
    fn active_token_at_cursor(&self) -> Option<(SuggestionMode, String, usize, usize)> {
        let value = self.input.value();
        let cursor = self.input.cursor();

        if cursor == 0 || value.is_empty() {
            return None;
        }

        let bytes = value.as_bytes();

        // Scan backwards from cursor to find the start of the current "word"
        let mut start = cursor;
        while start > 0 && bytes[start - 1] != b' ' {
            start -= 1;
        }

        // The token text from start to cursor
        let token = &value[start..cursor];

        if let Some(partial) = token.strip_prefix('@') {
            Some((SuggestionMode::Projects, partial.to_string(), start, cursor))
        } else if let Some(partial) = token.strip_prefix('#') {
            Some((SuggestionMode::Tags, partial.to_string(), start, cursor))
        } else {
            token
                .strip_prefix('!')
                .map(|partial| (SuggestionMode::Priorities, partial.to_string(), start, cursor))
        }
    }

    /// Updates the suggestion list based on the current token at the cursor.
    ///
    /// Called after every keystroke. Populates `suggestions` and sets
    /// `suggestion_mode` based on the active `@` or `#` token.
    fn update_suggestions(&mut self) {
        match self.active_token_at_cursor() {
            Some((SuggestionMode::Projects, partial, _, _)) => {
                let lower = partial.to_lowercase();
                let mut matches: Vec<String> = Vec::new();

                if lower.is_empty() {
                    // Show all projects
                    matches = self.projects.iter().map(|p| p.name.clone()).collect();
                } else {
                    // Exact matches first
                    for p in &self.projects {
                        if p.name.to_lowercase() == lower {
                            matches.push(p.name.clone());
                        }
                    }
                    // Prefix matches
                    for p in &self.projects {
                        if p.name.to_lowercase().starts_with(&lower)
                            && !matches.contains(&p.name)
                        {
                            matches.push(p.name.clone());
                        }
                    }
                    // Substring matches
                    for p in &self.projects {
                        if p.name.to_lowercase().contains(&lower)
                            && !matches.contains(&p.name)
                        {
                            matches.push(p.name.clone());
                        }
                    }
                }

                self.suggestions = matches;
                self.suggestion_mode = SuggestionMode::Projects;
                self.selected_suggestion = if self.suggestions.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            Some((SuggestionMode::Tags, partial, _, _)) => {
                let lower = partial.to_lowercase();
                let already_parsed: Vec<String> = self
                    .parsed
                    .tags
                    .iter()
                    .map(|t| t.to_lowercase())
                    .collect();

                // Build candidate list: project-scoped tags first, then others
                let mut scoped_tags: Vec<String> = Vec::new();
                let mut other_tags: Vec<String> = Vec::new();

                // Get project-scoped tags if a project is selected
                let project_id = self
                    .explicit_project
                    .as_ref()
                    .or(self.matched_project.as_ref())
                    .map(|p| p.id.clone());

                let scoped_tag_set: Vec<String> = project_id
                    .as_ref()
                    .and_then(|pid| self.project_tag_map.get(pid))
                    .cloned()
                    .unwrap_or_default();

                for tag in &self.all_tags {
                    // Skip tags already in the input
                    if already_parsed.contains(&tag.name.to_lowercase()) {
                        continue;
                    }

                    if scoped_tag_set.iter().any(|s| s.to_lowercase() == tag.name.to_lowercase()) {
                        scoped_tags.push(tag.name.clone());
                    } else {
                        other_tags.push(tag.name.clone());
                    }
                }

                // Filter by partial
                let filter = |tags: &[String]| -> Vec<String> {
                    if lower.is_empty() {
                        tags.to_vec()
                    } else {
                        tags.iter()
                            .filter(|t| t.to_lowercase().contains(&lower))
                            .cloned()
                            .collect()
                    }
                };

                let mut matches = filter(&scoped_tags);
                matches.extend(filter(&other_tags));

                self.suggestions = matches;
                self.suggestion_mode = SuggestionMode::Tags;
                self.selected_suggestion = if self.suggestions.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            Some((SuggestionMode::Priorities, partial, _, _)) => {
                // Priority options: !1 Urgent, !2 High, !3 Medium, !4 Low
                let all_priorities = vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                ];

                let matches: Vec<String> = if partial.is_empty() {
                    all_priorities
                } else {
                    all_priorities
                        .into_iter()
                        .filter(|p| p.starts_with(&partial))
                        .collect()
                };

                self.suggestions = matches;
                self.suggestion_mode = SuggestionMode::Priorities;
                self.selected_suggestion = if self.suggestions.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            _ => {
                self.suggestions.clear();
                self.selected_suggestion = None;
                self.suggestion_mode = SuggestionMode::None;
            }
        }
    }

    /// Fuzzy matches a project name against available projects.
    ///
    /// Matching priority: exact (case-insensitive) → prefix → substring.
    fn fuzzy_match_project(&self, name: &str) -> Option<Project> {
        let lower = name.to_lowercase();

        // Exact match (case-insensitive)
        if let Some(p) = self
            .projects
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
        {
            return Some(p.clone());
        }

        // Prefix match
        if let Some(p) = self
            .projects
            .iter()
            .find(|p| p.name.to_lowercase().starts_with(&lower))
        {
            return Some(p.clone());
        }

        // Substring match
        if let Some(p) = self
            .projects
            .iter()
            .find(|p| p.name.to_lowercase().contains(&lower))
        {
            return Some(p.clone());
        }

        None
    }

    /// Accepts the currently selected suggestion.
    ///
    /// For projects: removes the `@token` from input and sets `explicit_project`.
    /// For tags: replaces `#partial` with `#exactname` in input.
    fn accept_suggestion(&mut self) {
        let selected_idx = match self.selected_suggestion {
            Some(idx) if idx < self.suggestions.len() => idx,
            _ => return,
        };

        let selected = self.suggestions[selected_idx].clone();

        match self.suggestion_mode {
            SuggestionMode::Projects => {
                // Find the project
                if let Some(project) = self.projects.iter().find(|p| p.name == selected) {
                    self.explicit_project = Some(project.clone());
                }

                // Remove the @token from input (including trailing space if any)
                if let Some((_, _, start, end)) = self.active_token_at_cursor() {
                    self.replace_token_in_input(start, end, "");
                }

                self.suggestions.clear();
                self.selected_suggestion = None;
                self.suggestion_mode = SuggestionMode::None;
                self.reparse();
            }
            SuggestionMode::Tags => {
                // Replace #partial with #exactname
                if let Some((_, _, start, end)) = self.active_token_at_cursor() {
                    let replacement = format!("#{}", selected);
                    self.replace_token_in_input(start, end, &replacement);
                }

                self.suggestions.clear();
                self.selected_suggestion = None;
                self.suggestion_mode = SuggestionMode::None;
                self.reparse();
            }
            SuggestionMode::Priorities => {
                // Replace !partial with !N
                if let Some((_, _, start, end)) = self.active_token_at_cursor() {
                    let replacement = format!("!{}", selected);
                    self.replace_token_in_input(start, end, &replacement);
                }

                self.suggestions.clear();
                self.selected_suggestion = None;
                self.suggestion_mode = SuggestionMode::None;
                self.reparse();
            }
            SuggestionMode::None => {}
        }
    }

    /// Replaces a range in the input text and adjusts the cursor.
    fn replace_token_in_input(&mut self, start: usize, end: usize, replacement: &str) {
        let value = self.input.value().to_string();
        let mut new_value = String::new();
        new_value.push_str(&value[..start]);
        new_value.push_str(replacement);
        new_value.push_str(&value[end..]);

        // Trim leading/trailing spaces and collapse double spaces
        let new_value = new_value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let new_cursor = if replacement.is_empty() {
            // For project removal, put cursor at start position (adjusted for collapsed spaces)
            start.min(new_value.len())
        } else {
            // For tag replacement, put cursor after the replacement
            (start + replacement.len()).min(new_value.len())
        };

        self.input.set_value(&new_value);
        // set_value puts cursor at end; adjust to desired position
        self.input.move_home();
        for _ in 0..new_cursor {
            self.input.move_right();
        }
    }

    /// Creates a Task from the parsed fields.
    ///
    /// Prefers `explicit_project` (set via dropdown) over `matched_project` (fuzzy).
    /// Returns `None` if the title is empty.
    pub fn to_task(&self) -> Option<Task> {
        let title = self.parsed.title.trim();
        if title.is_empty() {
            return None;
        }

        let mut task = Task::new(title);
        task.priority = self.parsed.priority.unwrap_or(Priority::Medium);
        task.due_date = self.parsed.due_date;
        task.tags = self.parsed.tags.clone();

        let project = self.explicit_project.as_ref().or(self.matched_project.as_ref());
        if let Some(project) = project {
            task.project_id = Some(project.id.clone());
        }

        Some(task)
    }

    /// Converts the quick capture state into a pre-populated AddTaskDialog.
    ///
    /// Used when the user presses Tab to expand to the full dialog.
    /// Prefers `explicit_project` over `matched_project`.
    pub fn to_add_task_dialog(&self) -> AddTaskDialog {
        let mut dialog = AddTaskDialog::new().with_available_tags(self.all_tags.clone());

        // Set title
        if !self.parsed.title.is_empty() {
            dialog.title.set_value(&self.parsed.title);
        }

        // Set priority
        dialog.priority = self.parsed.priority.unwrap_or(Priority::Medium);

        // Set due date text
        if let Some(ref date_text) = self.parsed.due_date_text {
            dialog.due_date.set_value(date_text);
        }

        // Set project (prefer explicit selection)
        let project = self.explicit_project.as_ref().or(self.matched_project.as_ref());
        if let Some(project) = project {
            dialog.project_id = Some(project.id.clone());
        }

        // Set tags
        dialog.tags = crate::ui::tag_input::TagInput::with_tags(self.parsed.tags.clone());

        dialog
    }

    /// Renders the Quick Capture spotlight overlay.
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Dim background
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(theme::BG_DARK)),
            area,
        );

        // Calculate overlay dimensions
        let dialog_width = 60.min((area.width * 60 / 100).max(45)).max(45);
        let dialog_height: u16 = 6;

        // Position in upper 1/5th of screen
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = area.height / 5;
        let dialog_area = Rect::new(
            area.x + x,
            area.y + y,
            dialog_width.min(area.width.saturating_sub(x)),
            dialog_height.min(area.height.saturating_sub(y)),
        );

        // Dialog block
        let title_span = Span::styled(
            format!(" {} Quick Capture ", theme::icons::SPARKLE),
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        );
        let block = Block::default()
            .title(title_span)
            .borders(Borders::ALL)
            .border_set(ratatui::symbols::border::ROUNDED)
            .border_style(Style::default().fg(theme::PRIMARY_LIGHT))
            .style(Style::default().bg(theme::BG_ELEVATED));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        if inner.height < 4 || inner.width < 10 {
            return;
        }

        // Inner layout: input line, separator, title preview, metadata badges
        let chunks = Layout::vertical([
            Constraint::Length(1), // Input line
            Constraint::Length(1), // Separator
            Constraint::Length(1), // Title preview
            Constraint::Length(1), // Metadata badges
        ])
        .split(inner);

        // 1. Input line with syntax highlighting
        self.render_input_line(frame, chunks[0]);

        // 2. Separator
        let sep = "─".repeat(chunks[1].width as usize);
        frame.render_widget(
            Paragraph::new(sep).style(Style::default().fg(theme::BORDER_MUTED)),
            chunks[1],
        );

        // 3. Title preview
        self.render_title_preview(frame, chunks[2]);

        // 4. Metadata badges
        self.render_badges(frame, chunks[3]);

        // 5. Suggestion dropdown (rendered last to overlay on top)
        if !self.suggestions.is_empty() {
            self.render_suggestions(frame, dialog_area);
        }
    }

    /// Renders the suggestion dropdown below the input line.
    ///
    /// The dropdown appears directly below the dialog, overlaying content beneath it.
    /// Shows up to 5 items with the selected item highlighted.
    fn render_suggestions(&self, frame: &mut Frame, dialog_area: Rect) {
        let max_items: usize = 5;
        let visible_count = self.suggestions.len().min(max_items);
        if visible_count == 0 {
            return;
        }

        // Position dropdown directly below the dialog
        let dropdown_height = visible_count as u16 + 2; // +2 for borders
        let dropdown_y = dialog_area.y + dialog_area.height;
        let screen = frame.area();

        // Don't render past screen bottom
        if dropdown_y + dropdown_height > screen.y + screen.height {
            return;
        }

        let dropdown_area = Rect::new(
            dialog_area.x,
            dropdown_y,
            dialog_area.width,
            dropdown_height,
        );

        // Clear the area first
        frame.render_widget(Clear, dropdown_area);

        // Dropdown border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(ratatui::symbols::border::ROUNDED)
            .border_style(Style::default().fg(theme::BORDER_MUTED))
            .style(Style::default().bg(theme::BG_ELEVATED));

        let inner = block.inner(dropdown_area);
        frame.render_widget(block, dropdown_area);

        // Determine scroll offset to keep selected item visible
        let selected = self.selected_suggestion.unwrap_or(0);
        let scroll_offset = if selected >= max_items {
            selected - max_items + 1
        } else {
            0
        };

        // Render items
        for (i, suggestion) in self
            .suggestions
            .iter()
            .skip(scroll_offset)
            .take(max_items)
            .enumerate()
        {
            let actual_idx = i + scroll_offset;
            let is_selected = self.selected_suggestion == Some(actual_idx);
            let y = inner.y + i as u16;

            if y >= inner.y + inner.height {
                break;
            }

            let item_area = Rect::new(inner.x, y, inner.width, 1);

            let (prefix, label, color) = match self.suggestion_mode {
                SuggestionMode::Projects => (
                    format!("{} ", theme::icons::DIAMOND),
                    suggestion.clone(),
                    theme::PROJECT,
                ),
                SuggestionMode::Tags => (
                    "#".to_string(),
                    suggestion.clone(),
                    theme::TAG,
                ),
                SuggestionMode::Priorities => {
                    match suggestion.as_str() {
                        "1" => (
                            format!("{} ", theme::icons::PRIORITY_URGENT),
                            "Urgent".to_string(),
                            theme::PRIORITY_URGENT,
                        ),
                        "2" => (
                            format!("{} ", theme::icons::PRIORITY_HIGH),
                            "High".to_string(),
                            theme::PRIORITY_HIGH,
                        ),
                        "3" => (
                            "● ".to_string(),
                            "Medium".to_string(),
                            theme::INFO,
                        ),
                        "4" => (
                            format!("{} ", theme::icons::PRIORITY_LOW),
                            "Low".to_string(),
                            theme::PRIORITY_LOW,
                        ),
                        _ => (
                            "".to_string(),
                            suggestion.clone(),
                            theme::TEXT_SECONDARY,
                        ),
                    }
                }
                SuggestionMode::None => (
                    "".to_string(),
                    suggestion.clone(),
                    theme::TEXT_SECONDARY,
                ),
            };

            let style = if is_selected {
                Style::default()
                    .bg(theme::PRIMARY)
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            let text = format!("{}{}", prefix, label);
            // Truncate to fit
            let max_len = inner.width as usize;
            let display: String = text.chars().take(max_len).collect();

            frame.render_widget(Paragraph::new(display).style(style), item_area);
        }
    }

    /// Renders the input line with syntax highlighting and cursor.
    fn render_input_line(&self, frame: &mut Frame, area: Rect) {
        let prompt_span = Span::styled("> ", Style::default().fg(theme::ACCENT));

        let input_value = self.input.value();
        if input_value.is_empty() {
            // Show placeholder
            let line = Line::from(vec![
                prompt_span,
                Span::styled(
                    "Task title @project #tag !priority due:date",
                    Style::default().fg(theme::TEXT_MUTED),
                ),
            ]);
            frame.render_widget(Paragraph::new(line), area);
        } else {
            // Syntax-highlighted input
            let mut spans = vec![prompt_span];
            let highlighted = highlight_input(input_value);
            spans.extend(highlighted);
            let line = Line::from(spans);
            frame.render_widget(Paragraph::new(line), area);
        }

        // Render cursor
        let cursor_x = area.x + 2 + self.input.cursor() as u16; // 2 for "> "
        if cursor_x < area.x + area.width {
            let cursor_char = if self.input.cursor() < input_value.len() {
                input_value.chars().nth(self.input.cursor()).unwrap_or(' ')
            } else {
                ' '
            };
            let buf = frame.buffer_mut();
            buf[(cursor_x, area.y)]
                .set_char(cursor_char)
                .set_style(Style::default().bg(theme::ACCENT).fg(ratatui::style::Color::Black));
        }
    }

    /// Renders the title preview line.
    fn render_title_preview(&self, frame: &mut Frame, area: Rect) {
        let title = self.parsed.title.trim();
        if title.is_empty() {
            let line = Paragraph::new(Span::styled(
                "Title will appear here...",
                Style::default().fg(theme::TEXT_MUTED),
            ));
            frame.render_widget(line, area);
        } else {
            let line = Paragraph::new(Span::styled(
                title,
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ));
            frame.render_widget(line, area);
        }
    }

    /// Renders the metadata badges line.
    fn render_badges(&self, frame: &mut Frame, area: Rect) {
        let mut spans: Vec<Span> = Vec::new();

        // Project badge (prefer explicit_project, fall back to parsed)
        if let Some(ref project) = self.explicit_project {
            spans.push(Span::styled(
                format!("{} {} ", theme::icons::DIAMOND, project.name),
                Style::default().fg(theme::PROJECT),
            ));
        } else if let Some(ref project_name) = self.parsed.project_name {
            if let Some(ref matched) = self.matched_project {
                spans.push(Span::styled(
                    format!("{} {} ", theme::icons::DIAMOND, matched.name),
                    Style::default().fg(theme::PROJECT),
                ));
            } else {
                spans.push(Span::styled(
                    format!("{} {}? ", theme::icons::DIAMOND, project_name),
                    Style::default().fg(theme::TEXT_MUTED),
                ));
            }
        }

        // Priority badge
        if let Some(priority) = self.parsed.priority {
            let (icon, label, color) = match priority {
                Priority::Urgent => (theme::icons::PRIORITY_URGENT, "Urgent", theme::PRIORITY_URGENT),
                Priority::High => (theme::icons::PRIORITY_HIGH, "High", theme::PRIORITY_HIGH),
                Priority::Medium => ("●", "Medium", theme::INFO),
                Priority::Low => (theme::icons::PRIORITY_LOW, "Low", theme::PRIORITY_LOW),
            };
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                format!("{} {} ", icon, label),
                Style::default().fg(color),
            ));
        }

        // Tag badges
        for tag in &self.parsed.tags {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                format!("#{} ", tag),
                Style::default().fg(theme::TAG),
            ));
        }

        // Due date badge
        if let Some(ref date_text) = self.parsed.due_date_text {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            let color = if self.parsed.due_date.is_some() {
                theme::INFO
            } else {
                theme::TEXT_MUTED
            };
            spans.push(Span::styled(
                format!("{} {} ", theme::icons::CHECKBOX_PROGRESS, date_text),
                Style::default().fg(color),
            ));
        }

        // Empty state hint
        if spans.is_empty() {
            spans.push(Span::styled(
                "@project #tag !priority due:date",
                Style::default().fg(theme::TEXT_DISABLED),
            ));
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

/// Syntax-highlights a quick capture input string into colored spans.
fn highlight_input(input: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut first = true;

    for word in input.split(' ') {
        if !first {
            spans.push(Span::raw(" "));
        }
        first = false;

        if word.starts_with('@') && word.len() > 1 {
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(theme::PROJECT),
            ));
        } else if word.starts_with('#') && word.len() > 1 {
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(theme::TAG),
            ));
        } else if word.starts_with('!') && matches!(word, "!1" | "!2" | "!3" | "!4") {
            let color = match word {
                "!1" => theme::PRIORITY_URGENT,
                "!2" => theme::PRIORITY_HIGH,
                "!3" => theme::INFO,
                _ => theme::PRIORITY_LOW,
            };
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(color),
            ));
        } else if word.starts_with("due:") && word.len() > 4 {
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(theme::INFO),
            ));
        } else {
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(theme::TEXT_PRIMARY),
            ));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // ─── Parser tests ────────────────────────────────────────────────────

    #[test]
    fn test_parse_title_only() {
        let result = parse_capture_input("Buy groceries");
        assert_eq!(result.title, "Buy groceries");
        assert!(result.project_name.is_none());
        assert!(result.tags.is_empty());
        assert!(result.priority.is_none());
        assert!(result.due_date_text.is_none());
    }

    #[test]
    fn test_parse_with_project() {
        let result = parse_capture_input("Fix bug @Backend");
        assert_eq!(result.title, "Fix bug");
        assert_eq!(result.project_name.as_deref(), Some("Backend"));
    }

    #[test]
    fn test_parse_with_tags() {
        let result = parse_capture_input("Fix bug #urgent #backend");
        assert_eq!(result.title, "Fix bug");
        assert_eq!(result.tags, vec!["urgent", "backend"]);
    }

    #[test]
    fn test_parse_with_priority() {
        let result = parse_capture_input("Fix bug !1");
        assert_eq!(result.title, "Fix bug");
        assert_eq!(result.priority, Some(Priority::Urgent));
    }

    #[test]
    fn test_parse_priority_values() {
        assert_eq!(
            parse_capture_input("t !1").priority,
            Some(Priority::Urgent)
        );
        assert_eq!(parse_capture_input("t !2").priority, Some(Priority::High));
        assert_eq!(
            parse_capture_input("t !3").priority,
            Some(Priority::Medium)
        );
        assert_eq!(parse_capture_input("t !4").priority, Some(Priority::Low));
    }

    #[test]
    fn test_parse_invalid_priority_becomes_title() {
        let result = parse_capture_input("t !5");
        assert_eq!(result.title, "t !5");
        assert!(result.priority.is_none());
    }

    #[test]
    fn test_parse_with_due_date() {
        let result = parse_capture_input("Fix bug due:tomorrow");
        assert_eq!(result.title, "Fix bug");
        assert_eq!(result.due_date_text.as_deref(), Some("tomorrow"));
        assert!(result.due_date.is_some());
    }

    #[test]
    fn test_parse_full_syntax() {
        let result = parse_capture_input("Fix login bug @Backend #bug !1 due:tomorrow");
        assert_eq!(result.title, "Fix login bug");
        assert_eq!(result.project_name.as_deref(), Some("Backend"));
        assert_eq!(result.tags, vec!["bug"]);
        assert_eq!(result.priority, Some(Priority::Urgent));
        assert_eq!(result.due_date_text.as_deref(), Some("tomorrow"));
    }

    #[test]
    fn test_parse_escaped_at() {
        let result = parse_capture_input("Email \\@john about meeting");
        assert_eq!(result.title, "Email @john about meeting");
        assert!(result.project_name.is_none());
    }

    #[test]
    fn test_parse_escaped_hash() {
        let result = parse_capture_input("Fix issue \\#123");
        assert_eq!(result.title, "Fix issue #123");
        assert!(result.tags.is_empty());
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse_capture_input("");
        assert_eq!(result.title, "");
        assert!(result.project_name.is_none());
        assert!(result.tags.is_empty());
        assert!(result.priority.is_none());
    }

    #[test]
    fn test_parse_only_tokens() {
        let result = parse_capture_input("@Backend #bug !1 due:tomorrow");
        assert_eq!(result.title, "");
        assert_eq!(result.project_name.as_deref(), Some("Backend"));
        assert_eq!(result.tags, vec!["bug"]);
        assert_eq!(result.priority, Some(Priority::Urgent));
    }

    #[test]
    fn test_parse_empty_at_hash_ignored() {
        let result = parse_capture_input("hello @ # world");
        // Bare @ and # without a name are treated as title words
        assert_eq!(result.title, "hello @ # world");
    }

    // ─── Fuzzy matching tests ────────────────────────────────────────────

    #[test]
    fn test_fuzzy_match_exact() {
        let dialog = make_dialog_with_projects();
        let matched = dialog.fuzzy_match_project("Backend");
        assert_eq!(matched.map(|p| p.name), Some("Backend".to_string()));
    }

    #[test]
    fn test_fuzzy_match_prefix() {
        let dialog = make_dialog_with_projects();
        let matched = dialog.fuzzy_match_project("Back");
        assert_eq!(matched.map(|p| p.name), Some("Backend".to_string()));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let dialog = make_dialog_with_projects();
        let matched = dialog.fuzzy_match_project("backend");
        assert_eq!(matched.map(|p| p.name), Some("Backend".to_string()));
    }

    #[test]
    fn test_fuzzy_match_substring() {
        let dialog = make_dialog_with_projects();
        let matched = dialog.fuzzy_match_project("end");
        assert_eq!(matched.map(|p| p.name), Some("Backend".to_string()));
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let dialog = make_dialog_with_projects();
        let matched = dialog.fuzzy_match_project("xyz");
        assert!(matched.is_none());
    }

    // ─── Dialog key handling tests ───────────────────────────────────────

    #[test]
    fn test_handle_key_enter_submits() {
        let mut dialog = QuickCaptureDialog::new(vec![], vec![], &[]);
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), QuickCaptureAction::Submit);
    }

    #[test]
    fn test_handle_key_esc_cancels() {
        let mut dialog = QuickCaptureDialog::new(vec![], vec![], &[]);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), QuickCaptureAction::Cancel);
    }

    #[test]
    fn test_handle_key_tab_expands() {
        let mut dialog = QuickCaptureDialog::new(vec![], vec![], &[]);
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), QuickCaptureAction::ExpandToFull);
    }

    #[test]
    fn test_handle_key_char_input() {
        let mut dialog = QuickCaptureDialog::new(vec![], vec![], &[]);
        let key = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE);
        assert_eq!(dialog.handle_key(key), QuickCaptureAction::None);
        assert_eq!(dialog.input.value(), "H");
    }

    // ─── to_task tests ───────────────────────────────────────────────────

    #[test]
    fn test_to_task_empty_title_returns_none() {
        let dialog = QuickCaptureDialog::new(vec![], vec![], &[]);
        assert!(dialog.to_task().is_none());
    }

    #[test]
    fn test_to_task_with_parsed_data() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix bug @Backend #urgent !1 due:tomorrow");
        dialog.reparse();

        let task = dialog.to_task().unwrap();
        assert_eq!(task.title, "Fix bug");
        assert_eq!(task.priority, Priority::Urgent);
        assert_eq!(task.tags, vec!["urgent"]);
        assert!(task.project_id.is_some());
        assert!(task.due_date.is_some());
    }

    // ─── to_add_task_dialog tests ────────────────────────────────────────

    #[test]
    fn test_to_add_task_dialog_populates_fields() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix bug @Backend #urgent !2 due:tomorrow");
        dialog.reparse();

        let add_dialog = dialog.to_add_task_dialog();
        assert_eq!(add_dialog.title.value(), "Fix bug");
        assert_eq!(add_dialog.priority, Priority::High);
        assert!(add_dialog.project_id.is_some());
    }

    // ─── Suggestion tests ─────────────────────────────────────────────

    #[test]
    fn test_active_token_at_cursor_project() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix @ba");
        // Cursor is at end (position 7), which is inside "@ba"
        let result = dialog.active_token_at_cursor();
        assert!(result.is_some());
        let (mode, partial, start, end) = result.unwrap();
        assert_eq!(mode, SuggestionMode::Projects);
        assert_eq!(partial, "ba");
        assert_eq!(start, 4); // "@ba" starts at index 4
        assert_eq!(end, 7);
    }

    #[test]
    fn test_active_token_at_cursor_tag() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix #ur");
        let result = dialog.active_token_at_cursor();
        assert!(result.is_some());
        let (mode, partial, _, _) = result.unwrap();
        assert_eq!(mode, SuggestionMode::Tags);
        assert_eq!(partial, "ur");
    }

    #[test]
    fn test_active_token_at_cursor_none() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix bug");
        let result = dialog.active_token_at_cursor();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_suggestions_projects() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix @b");
        dialog.reparse();
        dialog.update_suggestions();

        assert_eq!(dialog.suggestion_mode, SuggestionMode::Projects);
        assert!(dialog.suggestions.contains(&"Backend".to_string()));
        assert!(!dialog.suggestions.contains(&"Frontend".to_string()));
    }

    #[test]
    fn test_update_suggestions_tags() {
        let tags = vec![
            Tag { id: "1".to_string(), name: "urgent".to_string() },
            Tag { id: "2".to_string(), name: "bug".to_string() },
            Tag { id: "3".to_string(), name: "feature".to_string() },
        ];
        let mut dialog = QuickCaptureDialog::new(vec![], tags, &[]);
        dialog.input.set_value("Fix #urg");
        dialog.reparse();
        dialog.update_suggestions();

        assert_eq!(dialog.suggestion_mode, SuggestionMode::Tags);
        assert!(dialog.suggestions.contains(&"urgent".to_string()));
        assert_eq!(dialog.suggestions.len(), 1); // Only "urgent" matches "urg"
    }

    #[test]
    fn test_update_suggestions_tags_project_scoped() {
        let projects = vec![Project::new("Backend")];
        let project_id = projects[0].id.clone();
        let tags = vec![
            Tag { id: "1".to_string(), name: "api".to_string() },
            Tag { id: "2".to_string(), name: "frontend".to_string() },
            Tag { id: "3".to_string(), name: "auth".to_string() },
        ];
        // Create a task in the project with the "api" tag
        let mut task = Task::new("API task");
        task.project_id = Some(project_id.clone());
        task.tags = vec!["api".to_string()];

        let mut dialog = QuickCaptureDialog::new(projects, tags, &[task]);
        // Set explicit project so scoping works
        dialog.explicit_project = Some(Project::new("Backend"));
        dialog.explicit_project.as_mut().unwrap().id = project_id;

        dialog.input.set_value("Fix #a");
        dialog.reparse();
        dialog.update_suggestions();

        // "api" should appear first (project-scoped), then "auth"
        assert_eq!(dialog.suggestion_mode, SuggestionMode::Tags);
        assert!(dialog.suggestions.len() >= 2);
        assert_eq!(dialog.suggestions[0], "api"); // scoped tag first
        assert_eq!(dialog.suggestions[1], "auth"); // non-scoped match
    }

    #[test]
    fn test_accept_project_removes_token() {
        let mut dialog = make_dialog_with_projects();
        // Type "@Back"
        dialog.input.set_value("Fix @Back bug");
        // Position cursor after "@Back" (position 9)
        dialog.input.move_home();
        for _ in 0..9 {
            dialog.input.move_right();
        }
        dialog.reparse();
        dialog.update_suggestions();

        assert!(!dialog.suggestions.is_empty());
        assert_eq!(dialog.suggestion_mode, SuggestionMode::Projects);

        // Accept the suggestion (Backend)
        dialog.accept_suggestion();

        // @Back should be removed from input
        assert!(dialog.explicit_project.is_some());
        assert_eq!(dialog.explicit_project.as_ref().unwrap().name, "Backend");
        // Input should not contain @Back anymore
        assert!(!dialog.input.value().contains("@Back"));
        assert!(dialog.input.value().contains("Fix"));
        assert!(dialog.input.value().contains("bug"));
    }

    #[test]
    fn test_accept_tag_replaces_token() {
        let tags = vec![
            Tag { id: "1".to_string(), name: "urgent".to_string() },
        ];
        let mut dialog = QuickCaptureDialog::new(vec![], tags, &[]);
        dialog.input.set_value("Fix #ur bug");
        // Position cursor after "#ur" (position 6)
        dialog.input.move_home();
        for _ in 0..6 {
            dialog.input.move_right();
        }
        dialog.reparse();
        dialog.update_suggestions();

        assert!(!dialog.suggestions.is_empty());
        assert_eq!(dialog.suggestion_mode, SuggestionMode::Tags);

        dialog.accept_suggestion();

        // #ur should be replaced with #urgent
        assert!(dialog.input.value().contains("#urgent"));
        assert!(!dialog.input.value().contains("#ur "));
    }

    #[test]
    fn test_tab_accepts_suggestion() {
        let mut dialog = make_dialog_with_projects();
        // Type "@B" to trigger suggestions
        let key_at = KeyEvent::new(KeyCode::Char('@'), KeyModifiers::NONE);
        dialog.handle_key(key_at);
        let key_b = KeyEvent::new(KeyCode::Char('B'), KeyModifiers::NONE);
        dialog.handle_key(key_b);

        assert!(!dialog.suggestions.is_empty());

        // Tab should accept suggestion instead of expanding
        let key_tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = dialog.handle_key(key_tab);
        assert_eq!(action, QuickCaptureAction::None); // Not ExpandToFull
        assert!(dialog.explicit_project.is_some());
    }

    #[test]
    fn test_explicit_project_cleared_on_new_at() {
        let mut dialog = make_dialog_with_projects();
        // Set an explicit project
        dialog.explicit_project = Some(dialog.projects[0].clone());

        // Type "@F" to start a new project token
        dialog.input.set_value("Fix @F");
        dialog.reparse();

        // The explicit project should be cleared because a new @token appeared
        assert!(dialog.explicit_project.is_none());
    }

    // ─── Priority suggestion tests ──────────────────────────────────────

    #[test]
    fn test_active_token_at_cursor_priority() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix !");
        let result = dialog.active_token_at_cursor();
        assert!(result.is_some());
        let (mode, partial, start, end) = result.unwrap();
        assert_eq!(mode, SuggestionMode::Priorities);
        assert_eq!(partial, "");
        assert_eq!(start, 4); // "!" starts at index 4
        assert_eq!(end, 5);
    }

    #[test]
    fn test_update_suggestions_priorities_all() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix !");
        dialog.reparse();
        dialog.update_suggestions();

        assert_eq!(dialog.suggestion_mode, SuggestionMode::Priorities);
        assert_eq!(dialog.suggestions.len(), 4);
        assert_eq!(dialog.suggestions, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_update_suggestions_priorities_filtered() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix !2");
        dialog.reparse();
        dialog.update_suggestions();

        assert_eq!(dialog.suggestion_mode, SuggestionMode::Priorities);
        assert_eq!(dialog.suggestions.len(), 1);
        assert_eq!(dialog.suggestions[0], "2");
    }

    #[test]
    fn test_accept_priority_replaces_token() {
        let mut dialog = make_dialog_with_projects();
        dialog.input.set_value("Fix ! bug");
        // Position cursor after "!" (position 5)
        dialog.input.move_home();
        for _ in 0..5 {
            dialog.input.move_right();
        }
        dialog.reparse();
        dialog.update_suggestions();

        assert_eq!(dialog.suggestion_mode, SuggestionMode::Priorities);
        // Select "1" (Urgent) - it's the first item
        dialog.accept_suggestion();

        // "!" should be replaced with "!1"
        assert!(dialog.input.value().contains("!1"));
        assert!(dialog.input.value().contains("Fix"));
        assert!(dialog.input.value().contains("bug"));
    }

    #[test]
    fn test_tab_accepts_priority_suggestion() {
        let mut dialog = make_dialog_with_projects();
        // Type "!" to trigger priority suggestions
        let key_bang = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
        dialog.handle_key(key_bang);

        assert_eq!(dialog.suggestion_mode, SuggestionMode::Priorities);
        assert_eq!(dialog.suggestions.len(), 4);

        // Navigate down to select "2" (High)
        let key_down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        dialog.handle_key(key_down);
        assert_eq!(dialog.selected_suggestion, Some(1));

        // Tab should accept the priority suggestion
        let key_tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let action = dialog.handle_key(key_tab);
        assert_eq!(action, QuickCaptureAction::None); // Not ExpandToFull
        assert!(dialog.input.value().contains("!2"));
    }

    // ─── Test helpers ────────────────────────────────────────────────────

    fn make_dialog_with_projects() -> QuickCaptureDialog {
        let projects = vec![
            Project::new("Backend"),
            Project::new("Frontend"),
            Project::new("Design"),
        ];
        QuickCaptureDialog::new(projects, vec![], &[])
    }
}
