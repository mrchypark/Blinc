#![cfg(unix)]

use blinc_recorder::{start_local_server_named, RecordingConfig, SharedRecordingSession};
use serde_json::Value;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn unique_app_name(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    format!("blinc-{label}-{}-{nanos}", std::process::id())
}

fn read_len_prefixed(stream: &mut UnixStream) -> Vec<u8> {
    let mut len_bytes = [0u8; 4];
    stream
        .read_exact(&mut len_bytes)
        .expect("length prefix should be readable");
    let len = u32::from_le_bytes(len_bytes) as usize;
    let mut payload = vec![0u8; len];
    stream
        .read_exact(&mut payload)
        .expect("payload should be readable");
    payload
}

fn write_len_prefixed(stream: &mut UnixStream, payload: &[u8]) {
    let len = payload.len() as u32;
    stream
        .write_all(&len.to_le_bytes())
        .expect("length prefix should be writable");
    stream
        .write_all(payload)
        .expect("payload should be writable");
}

fn read_json_message(stream: &mut UnixStream) -> Value {
    serde_json::from_slice(&read_len_prefixed(stream)).expect("message should be valid json")
}

fn request_export(stream: &mut UnixStream) {
    write_len_prefixed(stream, br#"{"type":"request_export"}"#);
}

fn measure_one_shot_fetch(stream: &mut UnixStream) -> Duration {
    let start = Instant::now();
    let hello = read_json_message(stream);
    assert_eq!(hello["type"], "hello");
    request_export(stream);
    let export = read_json_message(stream);
    assert_eq!(export["type"], "export");
    start.elapsed()
}

fn format_duration_stats(label: &str, samples: &mut [Duration]) -> String {
    samples.sort_unstable();
    let len = samples.len();
    let min = samples.first().copied().unwrap_or_default();
    let median = samples[len / 2];
    let p95_index = ((len.saturating_sub(1) as f64) * 0.95).round() as usize;
    let p95 = samples[p95_index.min(len - 1)];
    let max = samples.last().copied().unwrap_or_default();
    format!("{label}: n={len}, min={min:?}, median={median:?}, p95={p95:?}, max={max:?}")
}

#[test]
fn one_shot_attach_returns_empty_export_before_any_events() {
    let session = Arc::new(SharedRecordingSession::new(RecordingConfig::minimal()));
    let handle = start_local_server_named(unique_app_name("empty-export"), session)
        .expect("server should start");
    let socket_path = handle.socket_path().clone();

    let mut stream = UnixStream::connect(&socket_path).expect("client should connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("read timeout should be set");

    let hello = read_json_message(&mut stream);
    assert_eq!(hello["type"], "hello");

    request_export(&mut stream);
    let export = read_json_message(&mut stream);
    assert_eq!(export["type"], "export");
    assert_eq!(export["export"]["events"].as_array().unwrap().len(), 0);
    assert_eq!(export["export"]["snapshots"].as_array().unwrap().len(), 0);
    assert_eq!(
        export["export"]["trace_entries"].as_array().unwrap().len(),
        0
    );

    handle.shutdown();
    handle.join();
}

#[test]
fn one_shot_attach_stays_responsive_after_hello_during_busy_session() {
    let session = Arc::new(SharedRecordingSession::new(RecordingConfig::debug()));
    session.start();

    let busy_session = session.clone();
    let busy_thread = std::thread::spawn(move || {
        for _ in 0..64 {
            busy_session.record_event(blinc_recorder::RecordedEvent::WindowFocus(true));
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    let handle = start_local_server_named(unique_app_name("busy-export"), session)
        .expect("server should start");
    let socket_path = handle.socket_path().clone();

    let mut stream = UnixStream::connect(&socket_path).expect("client should connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("read timeout should be set");

    let hello = read_json_message(&mut stream);
    assert_eq!(hello["type"], "hello");

    std::thread::sleep(Duration::from_millis(20));
    let start = Instant::now();
    request_export(&mut stream);
    let export = read_json_message(&mut stream);
    let elapsed = start.elapsed();

    assert_eq!(export["type"], "export");
    assert!(
        elapsed < Duration::from_millis(80),
        "request_export should not be delayed by the server loop sleep; observed {:?}",
        elapsed
    );

    busy_thread.join().expect("busy thread should join");
    handle.shutdown();
    handle.join();
}

#[test]
fn server_restart_removes_stale_socket_from_prior_crash_like_run() {
    let app_name = unique_app_name("stale-restart");
    let stale_socket_path = std::path::PathBuf::from(format!("/tmp/blinc/{app_name}.sock"));

    if stale_socket_path.exists() {
        std::fs::remove_file(&stale_socket_path).expect("pre-existing stale socket should clear");
    }
    std::fs::create_dir_all(
        stale_socket_path
            .parent()
            .expect("default socket path should have a parent directory"),
    )
    .expect("socket parent directory should exist");

    let stale_listener =
        UnixListener::bind(&stale_socket_path).expect("stale socket should be creatable");
    drop(stale_listener);

    assert!(
        stale_socket_path.exists(),
        "dropping the listener should leave the socket path behind as stale state"
    );

    let session = Arc::new(SharedRecordingSession::new(RecordingConfig::minimal()));
    let handle =
        start_local_server_named(&app_name, session).expect("server should replace stale socket");
    let socket_path = handle.socket_path().clone();

    let mut stream =
        UnixStream::connect(&socket_path).expect("client should connect after restart");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("read timeout should be set");

    let hello = read_json_message(&mut stream);
    assert_eq!(hello["type"], "hello");

    request_export(&mut stream);
    let export = read_json_message(&mut stream);
    assert_eq!(export["type"], "export");

    handle.shutdown();
    handle.join();
}

#[test]
#[ignore = "manual benchmark used to choose the fixed debugger one-shot timeout"]
fn benchmark_one_shot_fetch_latency_profiles() {
    const ITERATIONS: usize = 50;

    let empty_session = Arc::new(SharedRecordingSession::new(RecordingConfig::minimal()));
    let empty_handle = start_local_server_named(unique_app_name("bench-empty"), empty_session)
        .expect("empty benchmark server should start");
    let empty_socket = empty_handle.socket_path().clone();
    let mut empty_samples = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let mut stream =
            UnixStream::connect(&empty_socket).expect("empty benchmark should connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("read timeout should be set");
        empty_samples.push(measure_one_shot_fetch(&mut stream));
    }
    empty_handle.shutdown();
    empty_handle.join();

    let mut busy_samples = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let busy_session = Arc::new(SharedRecordingSession::new(RecordingConfig::debug()));
        busy_session.start();
        let busy_handle =
            start_local_server_named(unique_app_name("bench-busy"), busy_session.clone())
                .expect("busy benchmark server should start");
        let busy_socket = busy_handle.socket_path().clone();
        let busy_thread = std::thread::spawn(move || {
            for _ in 0..64 {
                busy_session.record_event(blinc_recorder::RecordedEvent::WindowFocus(true));
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        let mut stream = UnixStream::connect(&busy_socket).expect("busy benchmark should connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("read timeout should be set");
        busy_samples.push(measure_one_shot_fetch(&mut stream));
        busy_thread
            .join()
            .expect("busy benchmark thread should join");
        busy_handle.shutdown();
        busy_handle.join();
    }

    let mut fresh_start_samples = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let session = Arc::new(SharedRecordingSession::new(RecordingConfig::minimal()));
        let handle = start_local_server_named(unique_app_name("bench-fresh"), session)
            .expect("fresh-start benchmark server should start");
        let socket = handle.socket_path().clone();
        let mut stream =
            UnixStream::connect(&socket).expect("fresh-start benchmark should connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("read timeout should be set");
        fresh_start_samples.push(measure_one_shot_fetch(&mut stream));
        handle.shutdown();
        handle.join();
    }

    println!("{}", format_duration_stats("empty", &mut empty_samples));
    println!("{}", format_duration_stats("busy", &mut busy_samples));
    println!(
        "{}",
        format_duration_stats("fresh-start", &mut fresh_start_samples)
    );
}
