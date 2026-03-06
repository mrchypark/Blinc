pub fn requires_display() -> bool {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
    }

    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        true
    }
}
