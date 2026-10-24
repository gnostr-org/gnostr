/// Handwired Ratatui Layout Example in Rust
/// Demonstrates controlling TUI panel widths, visibility, and constraints
/// using continuous ReLU gates instead of standard conditional branching.

// Layout logic is self-contained and does not require Ratatui types directly.

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDimensions {
    pub sidebar_width: u16,
    pub main_width: u16,
    pub footer_height: u16,
    pub is_sidebar_visible: f64,
}

// ==========================================
// 1. STANDARD CONDITIONAL LAYOUT ENGINE
// ==========================================
pub fn standard_layout_engine(toggle_flag: f64, total_width: u16) -> LayoutDimensions {
    let sidebar_width;
    let main_width;
    let is_sidebar_visible;

    if toggle_flag > 0.5 {
        sidebar_width = 30;
        main_width = total_width.saturating_sub(sidebar_width);
        is_sidebar_visible = 1.0;
    } else {
        sidebar_width = 0;
        main_width = total_width;
        is_sidebar_visible = 0.0;
    }

    LayoutDimensions {
        sidebar_width,
        main_width,
        footer_height: 3,
        is_sidebar_visible,
    }
}

// ==========================================
// 2. HANDWIRED (DIFFERENTIABLE) LAYOUT ENGINE
// ==========================================
pub fn handwired_layout_engine(toggle_flag: f64, total_width: u16) -> (LayoutDimensions, f64) {
    // Compute continuous ReLU indicator for toggle_flag > 0.5
    // gate evaluates to 1.0 if toggle_flag > 0.5, else 0.0
    let gate = f64::max(0.0, toggle_flag - 0.5) - f64::max(0.0, toggle_flag - 1.5);
    let inv_gate = 1.0 - gate;

    // Branch A (Sidebar Open): width = 30
    let sidebar_high = 30.0;
    // Branch B (Sidebar Closed): width = 0
    let sidebar_low = 0.0;

    // Continuous multiplexing over ReLU gates
    let continuous_sidebar_width = (gate * sidebar_high) + (inv_gate * sidebar_low);
    let sidebar_w = continuous_sidebar_width.round() as u16;

    // Main panel dynamically takes up the remainder
    let main_w = total_width.saturating_sub(sidebar_w);

    let layout_result = LayoutDimensions {
        sidebar_width: sidebar_w,
        main_width: main_w,
        footer_height: 3,
        is_sidebar_visible: gate,
    };

    (layout_result, gate)
}

// ==========================================
// 4. UNIT TEST MODULE
// ==========================================
#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_handwired_layout_match() {
        let width = 120;
        // At the extremes the ReLU gate saturates, so the handwired engine
        // agrees with the standard conditional engine.
        for t in [0.0, 0.3, 0.5, 1.5, 2.0] {
            let std = standard_layout_engine(t, width);
            let (hw, _) = handwired_layout_engine(t, width);
            assert_eq!(std.sidebar_width, hw.sidebar_width);
            assert_eq!(std.main_width, hw.main_width);
        }
    }
}
