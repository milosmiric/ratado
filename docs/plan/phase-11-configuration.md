# Phase 11: Configuration

**Goal:** Implement configuration file and theme support.

**Prerequisites:** Phase 1 (basic structure)

**Outcome:** Users can customize app behavior via config file.

---

## Story 11.1: Configuration File

**Priority:** Medium
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement configuration file loading and default creation.

### Tasks

- [ ] Create `src/models/config.rs`:
  - `Config` struct with all settings
  - Default values
  - TOML serialization/deserialization

- [ ] Config sections:
  ```toml
  [general]
  theme = "default"
  default_priority = "medium"
  date_format = "%Y-%m-%d"
  time_format = "%H:%M"
  week_start = "monday"  # or "sunday"

  [display]
  show_completed_tasks = true
  auto_archive_days = 7  # archive completed after N days, 0 = never

  [notifications]
  enabled = true
  sound = false
  desktop = true
  reminder_window_hours = 24
  ```

- [ ] Load config on startup:
  - Check `~/.config/ratado/config.toml`
  - Create default if not exists
  - Merge with defaults (for missing keys)

- [ ] Apply config to app behavior

### Code Sketch

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_priority")]
    pub default_priority: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_time_format")]
    pub time_format: String,
    #[serde(default = "default_week_start")]
    pub week_start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_completed_tasks: bool,
    #[serde(default = "default_archive_days")]
    pub auto_archive_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub sound: bool,
    #[serde(default = "default_true")]
    pub desktop: bool,
    #[serde(default = "default_reminder_hours")]
    pub reminder_window_hours: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            display: DisplayConfig::default(),
            notifications: NotificationConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default config
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs = ProjectDirs::from("", "", "ratado")
            .ok_or(ConfigError::NoConfigDir)?;
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("config.toml"))
    }
}

// Default value functions
fn default_theme() -> String { "default".to_string() }
fn default_priority() -> String { "medium".to_string() }
fn default_date_format() -> String { "%Y-%m-%d".to_string() }
fn default_time_format() -> String { "%H:%M".to_string() }
fn default_week_start() -> String { "monday".to_string() }
fn default_archive_days() -> u32 { 7 }
fn default_reminder_hours() -> u32 { 24 }
fn default_true() -> bool { true }
```

### Acceptance Criteria

- [ ] Config file created if not exists
- [ ] Default values are sensible
- [ ] Config loads on startup
- [ ] Missing keys use defaults
- [ ] Invalid config shows helpful error
- [ ] Settings affect app behavior

---

## Story 11.2: Theme Support

**Priority:** Low
**Estimate:** Medium
**Status:** [ ] Not Started

### Description

Implement color theme customization.

### Tasks

- [ ] Create `src/models/theme.rs`:
  - `Theme` struct with all color definitions
  - Default theme
  - Load custom theme from file

- [ ] Theme colors:
  ```toml
  [colors]
  background = "default"
  foreground = "white"
  selection_bg = "blue"
  selection_fg = "white"
  border = "gray"

  [colors.priority]
  urgent = "red"
  high = "yellow"
  medium = "default"
  low = "gray"

  [colors.status]
  pending = "default"
  in_progress = "cyan"
  completed = "gray"
  overdue = "red"

  [colors.ui]
  header = "cyan"
  footer = "gray"
  success = "green"
  warning = "yellow"
  error = "red"
  ```

- [ ] Color parsing:
  - Named colors: "red", "blue", "cyan", etc.
  - Hex colors: "#FF0000"
  - RGB: "rgb(255, 0, 0)"
  - "default" for terminal default

- [ ] Apply theme to all widgets

### Code Sketch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default)]
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_fg")]
    pub foreground: String,
    #[serde(default = "default_selection_bg")]
    pub selection_bg: String,
    #[serde(default)]
    pub priority: PriorityColors,
    #[serde(default)]
    pub status: StatusColors,
    #[serde(default)]
    pub ui: UiColors,
}

impl Theme {
    pub fn load(name: &str) -> Result<Self, ThemeError> {
        if name == "default" {
            return Ok(Theme::default());
        }

        let theme_path = Self::theme_path(name)?;
        if !theme_path.exists() {
            return Err(ThemeError::NotFound(name.to_string()));
        }

        let content = std::fs::read_to_string(&theme_path)?;
        let theme: Theme = toml::from_str(&content)?;
        Ok(theme)
    }

    pub fn theme_path(name: &str) -> Result<PathBuf, ThemeError> {
        let proj_dirs = ProjectDirs::from("", "", "ratado")
            .ok_or(ThemeError::NoConfigDir)?;
        Ok(proj_dirs.config_dir().join("themes").join(format!("{}.toml", name)))
    }

    pub fn get_color(&self, color_str: &str) -> Color {
        parse_color(color_str)
    }
}

fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "default" | "" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "white" => Color::White,
        "darkgray" | "darkgrey" => Color::DarkGray,
        s if s.starts_with('#') => {
            // Parse hex color
            if let Ok(rgb) = u32::from_str_radix(&s[1..], 16) {
                Color::Rgb(
                    ((rgb >> 16) & 0xFF) as u8,
                    ((rgb >> 8) & 0xFF) as u8,
                    (rgb & 0xFF) as u8,
                )
            } else {
                Color::Reset
            }
        }
        _ => Color::Reset,
    }
}

// In App, use theme for styling
impl App {
    pub fn style_for_priority(&self, priority: Priority) -> Style {
        let color = match priority {
            Priority::Urgent => &self.theme.colors.priority.urgent,
            Priority::High => &self.theme.colors.priority.high,
            Priority::Medium => &self.theme.colors.priority.medium,
            Priority::Low => &self.theme.colors.priority.low,
        };
        Style::default().fg(self.theme.get_color(color))
    }
}
```

### Example Custom Theme

```toml
# ~/.config/ratado/themes/nord.toml
[colors]
background = "#2E3440"
foreground = "#ECEFF4"
selection_bg = "#4C566A"
selection_fg = "#ECEFF4"
border = "#4C566A"

[colors.priority]
urgent = "#BF616A"
high = "#EBCB8B"
medium = "#ECEFF4"
low = "#4C566A"

[colors.status]
pending = "#ECEFF4"
in_progress = "#88C0D0"
completed = "#4C566A"
overdue = "#BF616A"

[colors.ui]
header = "#81A1C1"
footer = "#4C566A"
success = "#A3BE8C"
warning = "#EBCB8B"
error = "#BF616A"
```

### Acceptance Criteria

- [ ] Default theme matches specification
- [ ] Can create custom theme files
- [ ] Theme setting in config works
- [ ] Invalid theme falls back to default
- [ ] Hex colors work
- [ ] Named colors work
- [ ] Theme applies to all UI elements

---

## Phase 11 Checklist

Before moving to Phase 12, ensure:

- [ ] Both stories completed
- [ ] Config file loads correctly
- [ ] Default config created if missing
- [ ] Settings affect app behavior
- [ ] Theme customization works
- [ ] Documentation updated
