# Blinc Mobile Sensor Bridge Design (Android/iOS)

## Goal

Blinc 앱에서 Android/iOS 공통으로 다음 센서 데이터를 안정적으로 수집한다.

- GPS (location)
- Accelerometer
- Gyroscope
- (선택) Magnetometer, Barometer

핵심 목표는 "Rust 코어 중심 + 네이티브 수집기 최소화"이며, 현재 Blinc의 네이티브 브리지 구조(`native_call`)를 유지한다.

## Inputs Considered

- Existing bridge contract: `docs/native_bridge/blinc_native.idl`
- Existing platform bridges:
  - `extensions/blinc_platform_android/templates/BlincNativeBridge.kt`
  - `extensions/blinc_platform_ios/templates/BlincNativeBridge.swift`
- Existing mobile templates:
  - `toolchain/templates/rust/platforms/android/app/src/main/AndroidManifest.xml`
  - `toolchain/templates/rust/platforms/ios/BlincApp/Info.plist`
- External consultation:
  - `opencode run` with `@oracle` agent (session: `ses_38b8a7605ffewgLjjiefggO1U6`)

## Recommendation Summary

권장안은 Oracle 제안과 동일한 방향의 **Option A**다.

- 네이티브는 센서 수집/권한/OS lifecycle에 집중
- Rust는 정책/버퍼링/배치/저장/업로드를 담당
- FFI 경계는 샘플 단건이 아니라 **배치 프레임(frame)** 단위

현재 Blinc는 Rust에서 네이티브로 동기 호출하는 `native_call` 패턴이므로, 푸시 콜백 대신 **pull-drain 모델**을 채택한다.

## Architecture

```text
Android (Kotlin) / iOS (Swift)
  Sensor Collectors
    - GPS: FusedLocationProvider / CLLocationManager
    - IMU: SensorManager / CoreMotion
  Permission & Lifecycle Gate
  Native Ring Buffer (per sensor stream)
          |
          | native_call("sensor", "drain_frames", ...)
          v
Rust Core (new sensor module/crate)
  Session State Machine
  Adaptive Sampling Policy
  Normalizer (units/timestamps)
  Batcher + Compressor
  Local Store (store-and-forward)
  Uploader + Retry/Backoff
```

## Package Stack

### Rust (workspace)

필수:

- Existing: `serde`, `serde_json`, `tracing`, `tokio` (workspace already includes)
- Existing platform bridge crates:
  - `extensions/blinc_platform_android`
  - `extensions/blinc_platform_ios`

추가 권장:

- `smallvec` (already in workspace) for zero/low alloc frame buffers
- `parking_lot` for low-overhead buffer locks
- Optional persistence (phase 2):
  - `rusqlite` (explicit store-and-forward DB가 필요할 때)

### Android

- OS APIs:
  - `SensorManager` (accelerometer/gyro/magnetometer/barometer)
  - `FusedLocationProviderClient` (GPS/fused location)
- Gradle deps (권장):
  - `com.google.android.gms:play-services-location`
  - `androidx.work:work-runtime-ktx` (deferred upload)
  - `org.jetbrains.kotlinx:kotlinx-coroutines-android`

### iOS

- Frameworks:
  - `CoreLocation`
  - `CoreMotion`
- Optional:
  - `BackgroundTasks` (upload scheduling)

## Bridge Contract Additions (IDL proposal)

현재 `permissions.has_location()`까지만 존재하므로 센서 관련 namespace를 확장한다.

`namespace permissions`

- `request_location_when_in_use() -> Bool`
- `request_location_always() -> Bool`
- `has_motion() -> Bool`
- `request_motion() -> Bool`

`namespace sensor`

- `configure(config_json: String) -> Bool`
- `start(session_id: String) -> Bool`
- `stop(session_id: String) -> Bool`
- `status() -> String` (JSON)
- `drain_frames(max_frames: Int32) -> String` (JSON array)
- `clear_buffer() -> Void`

## Data Schema

권장 frame envelope:

- `seq: u64`
- `sensor_type: "gps" | "accel" | "gyro" | "mag" | "baro"`
- `time_monotonic_ns: u64`
- `time_unix_ms: i64`
- `sample_period_ns: u32`
- `accuracy: i32`
- `payload: []`

Payload examples:

- GPS: `lat, lon, alt_m, speed_mps, bearing_deg, h_acc_m`
- IMU: `x, y, z` (SI units)

## Permission and Lifecycle Policy

### Android

- Request order:
  1. `ACCESS_COARSE/FINE_LOCATION`
  2. (필요 시) `ACCESS_BACKGROUND_LOCATION`
  3. motion 관련 권한(API 레벨별 조건)
- 장시간 수집은 foreground service 전제로 설계
- 백그라운드 업로드는 WorkManager 사용

### iOS

- Request order:
  1. When-In-Use Location
  2. 필요 시 Always Location 승격
  3. Motion authorization 확인
- `Info.plist`에 설명 키 추가:
  - `NSLocationWhenInUseUsageDescription`
  - `NSLocationAlwaysAndWhenInUseUsageDescription` (필요 시)
  - `NSMotionUsageDescription`
- 배경 IMU는 제약이 크므로 기본 정책은 **foreground IMU + background GPS**

## Battery and Sampling Policy (default)

- Active session:
  - GPS 1 Hz
  - IMU 50 Hz
  - IMU frame flush 100-200ms
- Downshift triggers:
  - app background
  - low power mode
  - thermal warning
  - upload backlog growth
- Downshift action:
  - IMU 10-20 Hz or pause
  - GPS significant-change / reduced rate
  - upload defer

## Reliability / Privacy Guardrails

- Offline-first store-and-forward
- Exponential backoff + jitter
- Queue watermark 기반 drop/compact 정책
- User-visible collection status and consent controls
- 최소 수집 원칙(필요 없는 센서 비활성)

## Implementation Phases

### Phase 1 (MVP)

- IDL 확장 (`permissions`, `sensor`)
- Android/iOS native collector + ring buffer
- Rust pull-drain parser + normalized frame pipeline
- foreground session only

### Phase 2

- Adaptive sampling policy engine
- persistent queue + retry uploader
- debug metrics panel (drop count, queue depth, last upload)

### Phase 3

- Background capture optimization
- iOS/Android 차등 정책 고도화
- production observability (structured logs + telemetry)

## Validation Checklist

- Permission denial/partial grant 시 graceful fallback 동작
- 배터리 저전력 모드 전환 시 sampling 자동 하향
- 30분 연속 수집에서 OOM/noise/clock drift 없는지 확인
- 앱 재시작 후 queue 복구 및 중복 업로드 방지 확인

## Open Decision (Product)

iOS에서 앱 백그라운드 상태에서도 accel/gyro 연속 수집이 제품 요구사항인지 확정 필요.

- If `No` (권장 기본값): foreground IMU + background GPS 정책으로 진행
- If `Yes`: iOS 제약/심사 리스크를 반영한 별도 product expectation 필요
