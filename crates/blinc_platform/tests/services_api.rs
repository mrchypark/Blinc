use std::sync::{Mutex, OnceLock};

use blinc_core::native_bridge::{
    native_register, NativeBridgeError, NativeBridgeState, NativeValue,
};
use blinc_platform::app;
use blinc_platform::clipboard;
use blinc_platform::haptics;
use blinc_platform::permissions::{
    self, PermissionCapability, PermissionKind, PermissionRequestResult, PermissionStatus,
};

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
    assert_eq!(requested.status, PermissionStatus::Denied);
    assert!(requested.can_request_again);

    let granted = permissions::is_granted(PermissionKind::Microphone).expect("is_granted");
    assert!(granted);

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("permissions", "has_microphone");
    let _ = bridge.unregister("permissions", "request_microphone");
}

#[test]
fn permission_status_and_request_support_structured_payloads() {
    let _guard = test_lock().lock().expect("lock poisoned");
    ensure_bridge();

    native_register("permissions", "has_notifications", |_| {
        Ok(NativeValue::Json(
            serde_json::json!({
                "status": "provisional",
                "canRequest": true,
                "requiresSettingsRedirect": false,
                "supported": true
            })
            .to_string(),
        ))
    });
    native_register("permissions", "request_notifications", |_| {
        Ok(NativeValue::Json(
            serde_json::json!({
                "status": "provisional",
                "previousStatus": "not_determined",
                "canRequestAgain": true,
                "requiresSettingsRedirect": false
            })
            .to_string(),
        ))
    });
    native_register("permissions", "open_settings", |_| {
        Ok(NativeValue::Bool(true))
    });

    let capability = permissions::capability(PermissionKind::Notifications).expect("capability");
    assert_eq!(
        capability,
        PermissionCapability {
            status: PermissionStatus::Provisional,
            can_request: true,
            requires_settings_redirect: false,
            supported: true,
        }
    );

    let status = permissions::status(PermissionKind::Notifications).expect("status");
    assert_eq!(status, PermissionStatus::Provisional);

    let request = permissions::request(PermissionKind::Notifications).expect("request");
    assert_eq!(
        request,
        PermissionRequestResult {
            status: PermissionStatus::Provisional,
            previous_status: Some(PermissionStatus::NotDetermined),
            can_request_again: true,
            requires_settings_redirect: false,
        }
    );

    assert!(permissions::open_settings().expect("open_settings"));

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("permissions", "has_notifications");
    let _ = bridge.unregister("permissions", "request_notifications");
    let _ = bridge.unregister("permissions", "open_settings");
}

#[test]
fn permission_capability_supports_settings_redirect_and_unsupported_payloads() {
    let _guard = test_lock().lock().expect("lock poisoned");
    ensure_bridge();

    native_register("permissions", "has_camera", |_| {
        Ok(NativeValue::Json(
            serde_json::json!({
                "status": "restricted",
                "canRequest": false,
                "requiresSettingsRedirect": true,
                "supported": true
            })
            .to_string(),
        ))
    });
    native_register("permissions", "request_photos", |_| {
        Ok(NativeValue::Json(
            serde_json::json!({
                "status": "unknown",
                "previousStatus": "unknown",
                "canRequestAgain": false,
                "requiresSettingsRedirect": false
            })
            .to_string(),
        ))
    });

    let camera = permissions::capability(PermissionKind::Camera).expect("camera capability");
    assert_eq!(camera.status, PermissionStatus::Restricted);
    assert!(!camera.can_request);
    assert!(camera.requires_settings_redirect);
    assert!(camera.supported);

    let photos = permissions::request(PermissionKind::Photos).expect("photos request");
    assert_eq!(photos.status, PermissionStatus::Unknown);
    assert_eq!(photos.previous_status, Some(PermissionStatus::Unknown));
    assert!(!photos.can_request_again);
    assert!(!photos.requires_settings_redirect);

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("permissions", "has_camera");
    let _ = bridge.unregister("permissions", "request_photos");
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
fn app_and_haptics_wrappers_call_native_bridge() {
    let _guard = test_lock().lock().expect("lock poisoned");
    ensure_bridge();

    static CALLS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    let calls = CALLS.get_or_init(|| Mutex::new(Vec::new()));
    calls.lock().expect("calls lock").clear();

    let calls = CALLS.get_or_init(|| Mutex::new(Vec::new()));
    native_register("app", "open_url", move |args| {
        let url = args
            .first()
            .and_then(NativeValue::as_str)
            .unwrap_or_default()
            .to_string();
        calls
            .lock()
            .expect("calls lock")
            .push(format!("open_url:{url}"));
        Ok(NativeValue::Bool(true))
    });

    let calls = CALLS.get_or_init(|| Mutex::new(Vec::new()));
    native_register("app", "share_text", move |args| {
        let text = args
            .first()
            .and_then(NativeValue::as_str)
            .unwrap_or_default()
            .to_string();
        calls
            .lock()
            .expect("calls lock")
            .push(format!("share_text:{text}"));
        Ok(NativeValue::Bool(true))
    });

    let calls = CALLS.get_or_init(|| Mutex::new(Vec::new()));
    native_register("haptics", "selection", move |_| {
        calls
            .lock()
            .expect("calls lock")
            .push("selection".to_string());
        Ok(NativeValue::Void)
    });

    let calls = CALLS.get_or_init(|| Mutex::new(Vec::new()));
    native_register("haptics", "impact", move |args| {
        let style = args
            .first()
            .and_then(NativeValue::as_str)
            .unwrap_or_default()
            .to_string();
        calls
            .lock()
            .expect("calls lock")
            .push(format!("impact:{style}"));
        Ok(NativeValue::Void)
    });

    assert!(app::open_url("https://example.com").expect("open_url"));
    assert!(app::share_text("hello world").expect("share_text"));
    haptics::selection().expect("selection");
    haptics::impact("medium").expect("impact");

    assert_eq!(
        CALLS
            .get()
            .expect("calls")
            .lock()
            .expect("calls lock")
            .as_slice(),
        &[
            "open_url:https://example.com",
            "share_text:hello world",
            "selection",
            "impact:medium",
        ]
    );

    let bridge = NativeBridgeState::get();
    let _ = bridge.unregister("app", "open_url");
    let _ = bridge.unregister("app", "share_text");
    let _ = bridge.unregister("haptics", "selection");
    let _ = bridge.unregister("haptics", "impact");
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
