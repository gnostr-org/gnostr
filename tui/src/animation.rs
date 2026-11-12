//! Smooth value transitions for continuous layout parameters.
//!
//! A [`Transition`] linearly interpolates a `f64` value towards a target at a
//! given speed. It is useful for animating ReLU-gated layout parameters without
//! discrete jumps.

use std::time::Duration;

use ratatui::style::Color;

/// A value that moves smoothly towards a target over time.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transition {
    current: f64,
    target: f64,
    speed: f64,
}

impl Transition {
    /// Create a new transition starting at `initial` and moving `speed` units per second.
    pub fn new(initial: f64, speed: f64) -> Self {
        Self {
            current: initial,
            target: initial,
            speed: speed.max(0.0),
        }
    }

    /// Set the target value the transition should move towards.
    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    /// Toggle the target between `low` and `high` based on the current target.
    pub fn toggle_between(&mut self, low: f64, high: f64) {
        let midpoint = (low + high) / 2.0;
        self.target = if self.target > midpoint { low } else { high };
    }

    /// The current interpolated value.
    pub fn current(&self) -> f64 {
        self.current
    }

    /// The target value.
    pub fn target(&self) -> f64 {
        self.target
    }

    /// Returns `true` if the current value has reached the target.
    pub fn is_done(&self) -> bool {
        (self.current - self.target).abs() < f64::EPSILON
    }

    /// Snap the current value to the target immediately.
    pub fn snap(&mut self) {
        self.current = self.target;
    }

    /// Advance the transition by `dt`, moving the current value towards the target.
    pub fn update(&mut self, dt: Duration) {
        let step = self.speed * dt.as_secs_f64();
        if self.current < self.target {
            self.current = f64::min(self.current + step, self.target);
        } else if self.current > self.target {
            self.current = f64::max(self.current - step, self.target);
        }
    }
}

/// Smoothly animates between two RGB colors.
///
/// Each color channel is driven by its own [`Transition`]. Only
/// [`Color::Rgb`] values are interpolated; other variants are treated as
/// black.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorTransition {
    r: Transition,
    g: Transition,
    b: Transition,
}

impl ColorTransition {
    /// Create a new color transition starting at `start`.
    pub fn new(start: Color, speed: f64) -> Self {
        let (r, g, b) = rgb(start);
        Self {
            r: Transition::new(r, speed),
            g: Transition::new(g, speed),
            b: Transition::new(b, speed),
        }
    }

    /// Set the target color.
    pub fn set_target(&mut self, color: Color) {
        let (r, g, b) = rgb(color);
        self.r.set_target(r);
        self.g.set_target(g);
        self.b.set_target(b);
    }

    /// Advance the color transition by `dt`.
    pub fn update(&mut self, dt: Duration) {
        self.r.update(dt);
        self.g.update(dt);
        self.b.update(dt);
    }

    /// The current interpolated color.
    pub fn current(&self) -> Color {
        Color::Rgb(
            self.r.current() as u8,
            self.g.current() as u8,
            self.b.current() as u8,
        )
    }

    /// The target color.
    pub fn target(&self) -> Color {
        Color::Rgb(
            self.r.target() as u8,
            self.g.target() as u8,
            self.b.target() as u8,
        )
    }

    /// Returns `true` when the current color has reached the target.
    pub fn is_done(&self) -> bool {
        self.r.is_done() && self.g.is_done() && self.b.is_done()
    }
}

fn rgb(color: Color) -> (f64, f64, f64) {
    match color {
        Color::Rgb(r, g, b) => (f64::from(r), f64::from(g), f64::from(b)),
        _ => (0.0, 0.0, 0.0),
    }
}

/// Smoothly animates a transparency/alpha value between two colors.
///
/// The output is an opaque RGB color blended from `background` (alpha 0.0)
/// to `foreground` (alpha 1.0). This is useful for fade-in/fade-out effects
/// in terminals that don't support true alpha.
#[derive(Debug, Clone, PartialEq)]
pub struct AlphaTransition {
    transition: Transition,
}

impl AlphaTransition {
    /// Create a new alpha transition. `initial` is 0.0 (background) to 1.0
    /// (foreground).
    pub fn new(initial: f64, speed: f64) -> Self {
        Self {
            transition: Transition::new(initial.clamp(0.0, 1.0), speed),
        }
    }

    /// Set the target alpha, clamped to `[0.0, 1.0]`.
    pub fn set_target(&mut self, alpha: f64) {
        self.transition.set_target(alpha.clamp(0.0, 1.0));
    }

    /// Advance the alpha transition by `dt`.
    pub fn update(&mut self, dt: Duration) {
        self.transition.update(dt);
    }

    /// The current alpha value in `[0.0, 1.0]`.
    pub fn alpha(&self) -> f64 {
        self.transition.current().clamp(0.0, 1.0)
    }

    /// Blend `foreground` and `background` using the current alpha.
    ///
    /// Alpha 0.0 returns `background`; alpha 1.0 returns `foreground`.
    pub fn blend(&self, foreground: Color, background: Color) -> Color {
        blend(background, foreground, self.alpha())
    }

    /// Returns `true` when the alpha has reached the target.
    pub fn is_done(&self) -> bool {
        self.transition.is_done()
    }
}

/// Linearly blend between two RGB colors.
///
/// `t = 0.0` returns `a`, `t = 1.0` returns `b`.
pub fn blend(a: Color, b: Color, t: f64) -> Color {
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    let t = t.clamp(0.0, 1.0);
    Color::Rgb(
        lerp(ar, br, t) as u8,
        lerp(ag, bg, t) as u8,
        lerp(ab, bb, t) as u8,
    )
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_reaches_target() {
        let mut tr = Transition::new(0.0, 10.0);
        tr.set_target(5.0);
        tr.update(Duration::from_millis(100));
        assert!((tr.current() - 1.0).abs() < 1e-9);
        tr.update(Duration::from_secs(10));
        assert!((tr.current() - 5.0).abs() < 1e-9);
        assert!(tr.is_done());
    }

    #[test]
    fn transition_clamps_to_target() {
        let mut tr = Transition::new(0.0, 100.0);
        tr.set_target(1.0);
        tr.update(Duration::from_secs(10));
        assert!((tr.current() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn transition_toggle() {
        let mut tr = Transition::new(0.0, 1.0);
        tr.toggle_between(0.0, 1.0);
        assert_eq!(tr.target(), 1.0);
        tr.toggle_between(0.0, 1.0);
        assert_eq!(tr.target(), 0.0);
    }

    #[test]
    fn color_transition_reaches_target() {
        let mut ct = ColorTransition::new(Color::Rgb(0, 0, 0), 100.0);
        ct.set_target(Color::Rgb(100, 200, 50));
        ct.update(Duration::from_millis(100));
        assert!(matches!(ct.current(), Color::Rgb(_, _, _)));
        ct.update(Duration::from_secs(10));
        assert_eq!(ct.current(), Color::Rgb(100, 200, 50));
        assert!(ct.is_done());
    }

    #[test]
    fn alpha_transition_fades() {
        let mut at = AlphaTransition::new(0.0, 2.0);
        at.set_target(1.0);
        at.update(Duration::from_millis(200));
        let color = at.blend(Color::Rgb(255, 0, 0), Color::Rgb(0, 0, 0));
        assert!(matches!(color, Color::Rgb(r, 0, 0) if r > 0 && r < 255));
        at.update(Duration::from_secs(10));
        assert_eq!(
            at.blend(Color::Rgb(255, 0, 0), Color::Rgb(0, 0, 0)),
            Color::Rgb(255, 0, 0)
        );
    }

    #[test]
    fn blend_black_to_white() {
        assert_eq!(
            blend(Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255), 0.5),
            Color::Rgb(127, 127, 127)
        );
    }
}
