//! Animation and visual effects management.
//!
//! This module provides the [`AnimationState`] struct that wraps tachyonfx's
//! [`EffectManager`] to manage all visual effects in the application. Effects
//! are processed after normal widget rendering, modifying the terminal buffer
//! to create animations like fades, dissolves, and sweeps.
//!
//! ## Architecture
//!
//! Effects are identified by [`EffectKey`] for cancellable/unique effects.
//! The event loop runs at a fixed 16ms tick (60fps). The `needs_redraw` flag
//! in the main loop skips rendering when nothing has changed, keeping idle
//! CPU usage low.
//!
//! ## Delta Time Clamping
//!
//! When the app is idle (no redraws), the `last_frame` timestamp goes stale.
//! To prevent effects from completing instantly on their first frame after an
//! idle period, elapsed time is clamped to [`MAX_FRAME_DELTA`].

use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::{fx, Effect, EffectManager, Interpolation};

use super::theme;

/// Maximum time delta passed to effects per frame.
///
/// Prevents effects from completing instantly when the first frame after
/// an idle period has a large elapsed delta (e.g., app was idle for 2s,
/// user presses a key, a 500ms effect would otherwise complete in one frame).
/// Clamped to ~2 frames at 60fps.
const MAX_FRAME_DELTA: Duration = Duration::from_millis(32);

/// Unique identifiers for cancellable effects.
///
/// Effects spawned with a key can be cancelled or replaced. Spawning
/// a new effect with the same key automatically cancels the previous one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EffectKey {
    /// Startup splash screen animation
    #[default]
    Splash,
    /// Dialog open/close transition
    DialogTransition,
    /// Animation on a specific task row (by task ID)
    TaskRow(String),
    /// View change transition
    ViewTransition,
    /// Ambient effect on empty state artwork
    EmptyState,
}

/// Manages all visual effects and animations in the application.
///
/// Wraps tachyonfx's `EffectManager` with additional state for
/// tracking frame timing and per-area targeted effects (for task rows).
pub struct AnimationState {
    /// The tachyonfx effect manager for global screen effects
    manager: EffectManager<EffectKey>,
    /// Timestamp of the last frame, used to calculate elapsed time
    last_frame: Instant,
    /// Effects targeted at specific screen areas (e.g., individual task rows)
    targeted_effects: Vec<(Rect, Effect)>,
    /// Global toggle to enable/disable all animations
    pub enabled: bool,
}

impl std::fmt::Debug for AnimationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationState")
            .field("enabled", &self.enabled)
            .field("targeted_effects", &self.targeted_effects.len())
            .finish()
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationState {
    /// Creates a new AnimationState with animations enabled.
    pub fn new() -> Self {
        Self {
            manager: EffectManager::default(),
            last_frame: Instant::now(),
            targeted_effects: Vec::new(),
            enabled: true,
        }
    }

    /// Returns whether there are any active effects running.
    ///
    /// Used by the main loop to decide whether tick events need a redraw.
    pub fn has_active_effects(&self) -> bool {
        self.enabled && (self.manager.is_running() || !self.targeted_effects.is_empty())
    }

    /// Processes all active effects, applying them to the terminal buffer.
    ///
    /// This should be called inside `terminal.draw()` AFTER `ui::draw()`
    /// has rendered all widgets. Effects modify the already-rendered buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - The terminal buffer to modify
    /// * `area` - The full terminal area
    pub fn process(&mut self, buf: &mut Buffer, area: Rect) {
        if !self.enabled {
            return;
        }

        let elapsed = self.last_frame.elapsed().min(MAX_FRAME_DELTA);
        self.last_frame = Instant::now();

        // Process global effects
        self.manager.process_effects(elapsed, buf, area);

        // Process targeted effects (per-area)
        self.targeted_effects.retain_mut(|(rect, effect)| {
            // Only process if the target area is within bounds
            if rect.x < area.width && rect.y < area.height {
                effect.process(elapsed, buf, *rect);
            }
            !effect.done()
        });
    }

    /// Spawns a uniquely-keyed effect, cancelling any previous effect with the same key.
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for cancellation
    /// * `effect` - The effect to spawn
    pub fn spawn(&mut self, key: EffectKey, effect: Effect) {
        if !self.enabled {
            return;
        }
        self.manager.add_unique_effect(key, effect);
    }

    /// Spawns an anonymous effect that cannot be cancelled by key.
    ///
    /// # Arguments
    ///
    /// * `effect` - The effect to spawn
    pub fn spawn_anonymous(&mut self, effect: Effect) {
        if !self.enabled {
            return;
        }
        self.manager.add_effect(effect);
    }

    /// Spawns an effect targeted at a specific screen area.
    ///
    /// Used for task row animations where the effect should only
    /// modify a specific rectangular region of the screen.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to apply the effect to
    /// * `effect` - The effect to spawn
    pub fn spawn_targeted(&mut self, area: Rect, effect: Effect) {
        if !self.enabled {
            return;
        }
        self.targeted_effects.push((area, effect));
    }

    /// Cancels an active effect by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the effect to cancel
    pub fn cancel(&mut self, key: EffectKey) {
        self.manager.cancel_unique_effect(key);
    }

    /// Clears all targeted (per-area) effects.
    ///
    /// Call this when the screen layout changes significantly (e.g., dialog
    /// closes, tasks are deleted) to prevent stale effects from writing
    /// random characters to outdated screen coordinates.
    pub fn clear_targeted_effects(&mut self) {
        self.targeted_effects.clear();
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 2: Splash screen effects
    // ─────────────────────────────────────────────────────────────────────

    /// Starts the splash screen animation sequence.
    ///
    /// The sequence: coalesce (text materializes) → hold → fade to background.
    pub fn start_splash(&mut self) {
        let effect = fx::sequence(&[
            fx::coalesce((800, Interpolation::CubicOut)),
            fx::sleep(400),
            fx::fade_to(theme::BG_DARK, theme::BG_DARK, (500, Interpolation::CubicIn)),
        ]);
        self.spawn(EffectKey::Splash, effect);
    }

    /// Cancels the splash animation (e.g., on keypress to skip).
    pub fn cancel_splash(&mut self) {
        self.cancel(EffectKey::Splash);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 3: Dialog transition effects
    // ─────────────────────────────────────────────────────────────────────

    /// Starts a dialog open animation (fade-from effect).
    pub fn start_dialog_open(&mut self) {
        let effect = fx::fade_from(theme::BG_DARK, theme::BG_DARK, (200, Interpolation::CubicOut));
        self.spawn(EffectKey::DialogTransition, effect);
    }

    /// Starts a quick capture dialog open animation (quick fade).
    pub fn start_quick_capture_open(&mut self) {
        let effect = fx::fade_from(theme::BG_DARK, theme::BG_DARK, (150, Interpolation::CubicOut));
        self.spawn(EffectKey::DialogTransition, effect);
    }

    /// Handles dialog close by cancelling any in-progress open animation.
    ///
    /// No visual close effect is applied to avoid garbling text underneath.
    /// The dialog is dropped immediately by the caller.
    pub fn start_dialog_close(&mut self) {
        // Cancel any open animation that might still be running
        self.cancel(EffectKey::DialogTransition);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 4: Task micro-interaction effects
    // ─────────────────────────────────────────────────────────────────────

    /// Starts a green flash effect on a task row when completed.
    ///
    /// # Arguments
    ///
    /// * `area` - The task row's screen rectangle
    pub fn start_task_complete(&mut self, area: Rect) {
        let effect = fx::sequence(&[
            fx::fade_from(theme::SUCCESS, theme::SUCCESS, (300, Interpolation::CubicOut)),
            fx::fade_to(theme::BG_DARK, theme::BG_DARK, (500, Interpolation::CubicIn)),
        ]);
        self.spawn_targeted(area, effect);
    }

    /// Starts a dissolve effect on a task row before deletion.
    ///
    /// # Arguments
    ///
    /// * `area` - The task row's screen rectangle
    pub fn start_task_dissolve(&mut self, area: Rect) {
        let effect = fx::dissolve((300, Interpolation::CubicOut));
        self.spawn_targeted(area, effect);
    }

    /// Starts a coalesce effect on a newly added task row.
    ///
    /// The text materializes from random characters over 600ms,
    /// giving clear visual feedback that a new task was added.
    ///
    /// # Arguments
    ///
    /// * `area` - The task row's screen rectangle
    pub fn start_task_new(&mut self, area: Rect) {
        let effect = fx::coalesce((600, Interpolation::CubicOut));
        self.spawn_targeted(area, effect);
    }

    /// Starts a color flash on a task row after priority change.
    ///
    /// # Arguments
    ///
    /// * `area` - The task row's screen rectangle
    /// * `color` - The priority's semantic color to flash
    pub fn start_priority_flash(&mut self, area: Rect, color: Color) {
        let effect = fx::fade_from(color, color, (500, Interpolation::CubicOut));
        self.spawn_targeted(area, effect);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 5: Empty state ambient effects
    // ─────────────────────────────────────────────────────────────────────

    /// Starts a subtle ambient glow effect on the "All Done" empty state.
    pub fn start_empty_state_ambient(&mut self) {
        let effect = fx::ping_pong(fx::hsl_shift(
            Some([0.0, 0.0, 10.0]),  // fg: subtle lightness shift
            None,                     // bg: unchanged
            (2000, Interpolation::SineInOut),
        ));
        self.spawn(EffectKey::EmptyState, effect);
    }

    /// Cancels the empty state ambient effect.
    pub fn cancel_empty_state(&mut self) {
        self.cancel(EffectKey::EmptyState);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Phase 6: View transition effects
    // ─────────────────────────────────────────────────────────────────────

    /// Starts a view transition effect (quick fade).
    ///
    /// The `EffectKey::ViewTransition` uniqueness ensures rapid
    /// view switching auto-cancels in-progress transitions.
    pub fn start_view_transition(&mut self) {
        let effect = fx::fade_from(theme::BG_DARK, theme::BG_DARK, (150, Interpolation::CubicOut));
        self.spawn(EffectKey::ViewTransition, effect);
    }

}
