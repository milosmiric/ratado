//! Ratado Theme System
//!
//! A cohesive, modern color palette and styling system that gives Ratado
//! its distinctive visual identity. The theme uses a cool blue-violet gradient
//! as the signature look with warm accents for important elements.
//!
//! ## Design Philosophy
//!
//! - **Distinctive Identity**: Cool gradient tones create a memorable aesthetic
//! - **Semantic Colors**: Colors convey meaning (status, priority, category)
//! - **Clear Hierarchy**: Visual weight guides attention
//! - **Terminal-Native**: Optimized for terminal rendering

use ratatui::style::{Color, Modifier, Style};

// ═══════════════════════════════════════════════════════════════════════════════
// BRAND COLORS - The signature Ratado look
// ═══════════════════════════════════════════════════════════════════════════════

/// Primary brand color - A vibrant blue-violet
pub const PRIMARY: Color = Color::Rgb(99, 102, 241);      // Indigo-500

/// Primary light variant for hover/focus states
pub const PRIMARY_LIGHT: Color = Color::Rgb(129, 140, 248); // Indigo-400

/// Primary dark variant for borders and accents
pub const PRIMARY_DARK: Color = Color::Rgb(79, 70, 229);   // Indigo-600

/// Secondary brand color - A complementary teal
pub const SECONDARY: Color = Color::Rgb(20, 184, 166);     // Teal-500

/// Accent color - Warm amber for highlights
pub const ACCENT: Color = Color::Rgb(251, 191, 36);        // Amber-400

// ═══════════════════════════════════════════════════════════════════════════════
// SEMANTIC COLORS - Colors that convey meaning
// ═══════════════════════════════════════════════════════════════════════════════

/// Success/completed state - Fresh green
pub const SUCCESS: Color = Color::Rgb(34, 197, 94);        // Green-500

/// Warning state - Warm orange
pub const WARNING: Color = Color::Rgb(251, 146, 60);       // Orange-400

/// Error/urgent state - Bold red
pub const ERROR: Color = Color::Rgb(239, 68, 68);          // Red-500

/// Info state - Sky blue
pub const INFO: Color = Color::Rgb(56, 189, 248);          // Sky-400

// ═══════════════════════════════════════════════════════════════════════════════
// SURFACE COLORS - Backgrounds and containers
// ═══════════════════════════════════════════════════════════════════════════════

/// Main background - Deep dark
pub const BG_DARK: Color = Color::Rgb(15, 15, 25);

/// Elevated surface - Slightly lighter for panels
pub const BG_ELEVATED: Color = Color::Rgb(25, 25, 40);

/// Selection background - Subtle highlight
pub const BG_SELECTION: Color = Color::Rgb(45, 45, 70);

/// Hover background - Interactive state
pub const BG_HOVER: Color = Color::Rgb(55, 55, 85);

// ═══════════════════════════════════════════════════════════════════════════════
// BORDER COLORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Default border - Visible divider
pub const BORDER: Color = Color::Rgb(100, 100, 120);

/// Focused border - Uses primary color
pub const BORDER_FOCUSED: Color = PRIMARY_LIGHT;

/// Muted border - Subtle
pub const BORDER_MUTED: Color = Color::Rgb(55, 55, 75);

// ═══════════════════════════════════════════════════════════════════════════════
// TEXT COLORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Primary text - High contrast
pub const TEXT_PRIMARY: Color = Color::Rgb(245, 245, 250);

/// Secondary text - Medium contrast
pub const TEXT_SECONDARY: Color = Color::Rgb(160, 160, 180);

/// Muted text - Low contrast for metadata
pub const TEXT_MUTED: Color = Color::Rgb(100, 100, 120);

/// Completed/done text - Readable but subdued
pub const TEXT_COMPLETED: Color = Color::Rgb(130, 130, 150);

/// Disabled text - Very low contrast
pub const TEXT_DISABLED: Color = Color::Rgb(70, 70, 85);

// ═══════════════════════════════════════════════════════════════════════════════
// CATEGORY COLORS - For projects and tags
// ═══════════════════════════════════════════════════════════════════════════════

/// Project color - Soft blue
pub const PROJECT: Color = Color::Rgb(96, 165, 250);       // Blue-400

/// Tag color - Soft purple
pub const TAG: Color = Color::Rgb(192, 132, 252);          // Purple-400

// ═══════════════════════════════════════════════════════════════════════════════
// PRIORITY COLORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Urgent priority - Bold red
pub const PRIORITY_URGENT: Color = ERROR;

/// High priority - Warm orange
pub const PRIORITY_HIGH: Color = WARNING;

/// Normal priority - Default (no special color)
pub const PRIORITY_NORMAL: Color = TEXT_SECONDARY;

/// Low priority - Muted
pub const PRIORITY_LOW: Color = TEXT_MUTED;

// ═══════════════════════════════════════════════════════════════════════════════
// DUE DATE COLORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Overdue - Error red
pub const DUE_OVERDUE: Color = ERROR;

/// Due today - Warning amber
pub const DUE_TODAY: Color = ACCENT;

/// Due this week - Info blue
pub const DUE_WEEK: Color = INFO;

/// Due later - Muted
pub const DUE_LATER: Color = TEXT_MUTED;

// ═══════════════════════════════════════════════════════════════════════════════
// STATUS COLORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Pending status - Amber
pub const STATUS_PENDING: Color = ACCENT;

/// In progress status - Primary blue
pub const STATUS_IN_PROGRESS: Color = PRIMARY_LIGHT;

/// Completed status - Success green
pub const STATUS_COMPLETED: Color = SUCCESS;

/// Archived status - Muted
pub const STATUS_ARCHIVED: Color = TEXT_MUTED;

// ═══════════════════════════════════════════════════════════════════════════════
// ICONS & SYMBOLS - A consistent icon set
// ═══════════════════════════════════════════════════════════════════════════════

pub mod icons {
    //! Unicode icons and symbols for consistent visual language.

    // Checkbox states
    pub const CHECKBOX_EMPTY: &str = "○";
    pub const CHECKBOX_PROGRESS: &str = "◐";
    pub const CHECKBOX_DONE: &str = "●";
    pub const CHECKBOX_ARCHIVED: &str = "◌";

    // Priority indicators
    pub const PRIORITY_URGENT: &str = "▲";
    pub const PRIORITY_HIGH: &str = "△";
    pub const PRIORITY_LOW: &str = "▽";

    // Navigation & selection
    pub const ARROW_RIGHT: &str = "→";
    pub const ARROW_LEFT: &str = "←";
    pub const CHEVRON_RIGHT: &str = "›";
    pub const CHEVRON_DOWN: &str = "⌄";
    pub const SELECTOR: &str = "▸";
    pub const BULLET: &str = "•";

    // Category prefixes
    pub const PROJECT_PREFIX: &str = "@";
    pub const TAG_PREFIX: &str = "#";

    // Status indicators
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const WARNING_ICON: &str = "⚠";
    pub const INFO_ICON: &str = "ℹ";

    // Decorative
    pub const SPARKLE: &str = "✦";
    pub const STAR: &str = "★";
    pub const DIAMOND: &str = "◆";
    pub const CIRCLE: &str = "●";
    pub const DOT: &str = "·";

    // Box drawing enhancements
    pub const LINE_HORIZONTAL: &str = "─";
    pub const LINE_VERTICAL: &str = "│";
    pub const CORNER_TL: &str = "╭";
    pub const CORNER_TR: &str = "╮";
    pub const CORNER_BL: &str = "╰";
    pub const CORNER_BR: &str = "╯";

    // Progress
    pub const PROGRESS_EMPTY: &str = "░";
    pub const PROGRESS_PARTIAL: &str = "▒";
    pub const PROGRESS_FULL: &str = "█";
}

// ═══════════════════════════════════════════════════════════════════════════════
// STYLE PRESETS - Common style combinations
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns the style for primary/brand text.
pub fn primary_style() -> Style {
    Style::default().fg(PRIMARY_LIGHT)
}

/// Returns the style for headings/titles.
pub fn heading_style() -> Style {
    Style::default()
        .fg(TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Returns the style for subheadings.
pub fn subheading_style() -> Style {
    Style::default().fg(TEXT_SECONDARY)
}

/// Returns the style for muted/secondary text.
pub fn muted_style() -> Style {
    Style::default().fg(TEXT_MUTED)
}

/// Returns the style for disabled elements.
pub fn disabled_style() -> Style {
    Style::default()
        .fg(TEXT_DISABLED)
        .add_modifier(Modifier::DIM)
}

/// Returns the style for focused borders.
pub fn border_focused_style() -> Style {
    Style::default().fg(BORDER_FOCUSED)
}

/// Returns the style for unfocused borders.
pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

/// Returns the style for selection highlight.
pub fn selection_style() -> Style {
    Style::default()
        .bg(BG_SELECTION)
        .add_modifier(Modifier::BOLD)
}

/// Returns the style for keyboard shortcuts in help text.
pub fn keybind_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

/// Returns style for success messages.
pub fn success_style() -> Style {
    Style::default().fg(SUCCESS)
}

/// Returns style for warning messages.
pub fn warning_style() -> Style {
    Style::default().fg(WARNING)
}

/// Returns style for error messages.
pub fn error_style() -> Style {
    Style::default().fg(ERROR)
}

/// Returns style for info messages.
pub fn info_style() -> Style {
    Style::default().fg(INFO)
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRADIENT HELPERS - For creating visual depth
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a gradient effect for headers using block characters.
pub fn gradient_bar(width: usize) -> String {
    let chars = ['░', '▒', '▓', '█', '▓', '▒', '░'];
    let segment_width = width / chars.len();
    let mut result = String::new();

    for (i, &ch) in chars.iter().enumerate() {
        let count = if i == chars.len() / 2 {
            width - (segment_width * (chars.len() - 1))
        } else {
            segment_width
        };
        for _ in 0..count {
            result.push(ch);
        }
    }

    result
}

/// Returns a progress bar string representation.
pub fn progress_bar(progress: f32, width: usize) -> String {
    let filled = (progress * width as f32) as usize;
    let empty = width.saturating_sub(filled);

    format!(
        "{}{}",
        icons::PROGRESS_FULL.repeat(filled),
        icons::PROGRESS_EMPTY.repeat(empty)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0.5, 10), "█████░░░░░");
        assert_eq!(progress_bar(1.0, 5), "█████");
        assert_eq!(progress_bar(0.0, 5), "░░░░░");
    }

    #[test]
    fn test_gradient_bar() {
        let bar = gradient_bar(21);
        assert_eq!(bar.chars().count(), 21);
    }
}
