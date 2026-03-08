pub(crate) fn wait_frames_for_duration(wait_ms: u64, tick_ms: u64) -> u32 {
    if wait_ms == 0 {
        return 1;
    }

    let tick = tick_ms.max(1);
    let frames = wait_ms / tick + u64::from(wait_ms % tick != 0);
    frames.max(1).min(u64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::wait_frames_for_duration;

    #[test]
    fn wait_frames_clamps_before_narrowing_to_u32() {
        assert_eq!(
            wait_frames_for_duration(u64::from(u32::MAX) + 1, 1),
            u32::MAX
        );
    }

    #[test]
    fn wait_frames_uses_ceil_division_without_overflow() {
        assert_eq!(wait_frames_for_duration(u64::MAX, 2), u32::MAX);
        assert_eq!(wait_frames_for_duration(u64::MAX, 16), u32::MAX);
        assert_eq!(
            wait_frames_for_duration(u64::MAX, u64::from(u32::MAX) + 3),
            u32::MAX
        );
    }

    #[test]
    fn wait_frames_advances_one_frame_for_zero_duration_waits() {
        assert_eq!(wait_frames_for_duration(0, 16), 1);
        assert_eq!(wait_frames_for_duration(0, 0), 1);
    }
}
