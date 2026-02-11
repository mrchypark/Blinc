#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMode {
    Wide,
    Narrow,
}

pub const NARROW_BREAKPOINT_W: f32 = 900.0;

pub fn layout_mode(width: f32, _height: f32) -> LayoutMode {
    if width < NARROW_BREAKPOINT_W {
        LayoutMode::Narrow
    } else {
        LayoutMode::Wide
    }
}

pub fn parse_initial_selected(selected: Option<&str>, items_len: usize) -> usize {
    if items_len == 0 {
        return 0;
    }
    selected
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&i| i < items_len)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_mode_breakpoint() {
        assert_eq!(layout_mode(1200.0, 800.0), LayoutMode::Wide);
        assert_eq!(layout_mode(900.0, 800.0), LayoutMode::Wide);
        assert_eq!(layout_mode(899.0, 800.0), LayoutMode::Narrow);
    }

    #[test]
    fn test_parse_initial_selected() {
        assert_eq!(parse_initial_selected(None, 10), 0);
        assert_eq!(parse_initial_selected(Some("0"), 10), 0);
        assert_eq!(parse_initial_selected(Some("9"), 10), 9);
        assert_eq!(parse_initial_selected(Some("10"), 10), 0);
        assert_eq!(parse_initial_selected(Some(" 3 "), 10), 3);
        assert_eq!(parse_initial_selected(Some("nope"), 10), 0);
        assert_eq!(parse_initial_selected(Some("1"), 0), 0);
    }
}
