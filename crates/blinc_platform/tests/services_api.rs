use std::sync::{Mutex, OnceLock};

use blinc_core::native_bridge::{
    native_register, NativeBridgeError, NativeBridgeState, NativeValue,
};
use blinc_platform::clipboard;
use blinc_platform::permissions::{self, PermissionKind, PermissionStatus};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn ensure_bridge() {
    if !NativeBridgeState::is_initialized() {
        NativeBridgeState::init();
    }
}

#[test]
fn permission_status_and_request_roundtrip() {
    let _guard = test_lock().lock().expect("lock poisoned");
    ensure_bridge();

    native_register("permissions", "has_microphone", |_| {
        Ok(NativeValue::Bool(true))
    });
    native_register("permissions", "request_microphone", |_| {
        Ok(NativeValue::Bool(false))
    });

    let status = permissions::status(PermissionKind::Microphone).expect("status");
    assert_eq!(status, PermissionStatus::Granted);

    let requested = permissions::request(PermissionKind::Microphone).expect("request");
    assert_eq!(requested, PermissionStatus::Denied);

    let granted = permissions::is_granted(PermissionKind::Microphone).expect("is_granted");
    assert!(granted);

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("permissions", "has_microphone");
    let _ = bridge.unregister("permissions", "request_microphone");
}

#[test]
fn clipboard_wrapper_calls_native_bridge() {
    let _guard = test_lock().lock().expect("lock poisoned");
    ensure_bridge();

    static CLIPBOARD: OnceLock<Mutex<String>> = OnceLock::new();
    let clipboard_state = CLIPBOARD.get_or_init(|| Mutex::new(String::new()));

    native_register("clipboard", "copy", move |args| {
        let text = args
            .first()
            .and_then(NativeValue::as_str)
            .unwrap_or_default()
            .to_string();
        *clipboard_state.lock().expect("clipboard lock") = text;
        Ok(NativeValue::Void)
    });

    let clipboard_state = CLIPBOARD.get_or_init(|| Mutex::new(String::new()));
    native_register("clipboard", "paste", move |_| {
        Ok(NativeValue::String(
            clipboard_state.lock().expect("clipboard lock").clone(),
        ))
    });

    let clipboard_state = CLIPBOARD.get_or_init(|| Mutex::new(String::new()));
    native_register("clipboard", "has_content", move |_| {
        let has_content = !clipboard_state.lock().expect("clipboard lock").is_empty();
        Ok(NativeValue::Bool(has_content))
    });

    let clipboard_state = CLIPBOARD.get_or_init(|| Mutex::new(String::new()));
    native_register("clipboard", "clear", move |_| {
        clipboard_state.lock().expect("clipboard lock").clear();
        Ok(NativeValue::Void)
    });

    clipboard::copy("hello").expect("copy");
    assert_eq!(clipboard::paste().expect("paste"), "hello");
    assert!(clipboard::has_content().expect("has_content"));

    clipboard::clear().expect("clear");
    assert_eq!(clipboard::paste().expect("paste after clear"), "");
    assert!(!clipboard::has_content().expect("has_content after clear"));

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("clipboard", "copy");
    let _ = bridge.unregister("clipboard", "paste");
    let _ = bridge.unregister("clipboard", "has_content");
    let _ = bridge.unregister("clipboard", "clear");
}

#[test]
fn status_returns_unknown_on_missing_handler() {
    let _guard = test_lock().lock().expect("lock poisoned");
    ensure_bridge();

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("permissions", "has_camera");

    let status = permissions::status(PermissionKind::Camera).expect("status");
    assert_eq!(status, PermissionStatus::Unknown);

    let granted = permissions::is_granted(PermissionKind::Camera).expect("is_granted");
    assert!(!granted);

    let err =
        permissions::request(PermissionKind::Photos).expect_err("missing request_photos handler");
    assert!(matches!(
        err,
        blinc_platform::PlatformError::Bridge(NativeBridgeError::NotRegistered { .. })
    ));
}
