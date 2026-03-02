use std::time::Duration;

pub(crate) fn android_poll_timeout(
    needs_rebuild: bool,
    needs_redraw_next_frame: bool,
    has_tick_callbacks: bool,
    focused: bool,
) -> Duration {
    if needs_rebuild || needs_redraw_next_frame {
        Duration::ZERO
    } else if focused && has_tick_callbacks {
        Duration::from_millis(16)
    } else {
        Duration::from_millis(100)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn android_poll_timeout_prefers_zero_when_needs_frame() {
        assert_eq!(
            super::android_poll_timeout(true, false, true, true),
            Duration::ZERO
        );
        assert_eq!(
            super::android_poll_timeout(false, true, true, true),
            Duration::ZERO
        );
    }

    #[test]
    fn android_poll_timeout_uses_16ms_for_tick_callbacks_when_focused() {
        assert_eq!(
            super::android_poll_timeout(false, false, true, true),
            Duration::from_millis(16)
        );
    }

    #[test]
    fn android_poll_timeout_uses_100ms_when_idle() {
        assert_eq!(
            super::android_poll_timeout(false, false, false, true),
            Duration::from_millis(100)
        );
        assert_eq!(
            super::android_poll_timeout(false, false, true, false),
            Duration::from_millis(100)
        );
        assert_eq!(
            super::android_poll_timeout(false, false, false, false),
            Duration::from_millis(100)
        );
    }
}
