# Sensor API Packaging Decision (Blinc)

## Context

목표는 Android/iOS 센서를 대상으로 Blinc에서 **통일된 API**를 제공하는 것이다.

현재 구조 요약:

- 공통 추상화: `crates/blinc_platform`
- 플랫폼 구현: `extensions/blinc_platform_android`, `extensions/blinc_platform_ios`
- Rust ↔ native 호출: `crates/blinc_core/src/native_bridge.rs`
- 모바일 런타임 진입점: `crates/blinc_app/src/android.rs`, `crates/blinc_app/src/ios.rs`
- native bridge 샘플 구현: `extensions/blinc_platform_android/templates/BlincNativeBridge.kt`, `extensions/blinc_platform_ios/templates/BlincNativeBridge.swift`

## Options

### Option A: extensions만 확장

- 방식:
  - Android/iOS extension crate에 센서 API 추가
  - native bridge handler를 각 플랫폼에서 직접 구현
- 장점:
  - 초기 구현 속도 빠름
  - 플랫폼별 최적화/예외처리 자유도 높음
- 단점:
  - Rust 앱에서 공통 typed API가 생기지 않음
  - 정책(샘플링/배터리/배치) 중복 구현 위험
  - 테스트/리플레이 공통화 어려움

### Option B: `blinc_platform` trait 자체 확장

- 방식:
  - `crates/blinc_platform`에 Sensor trait/Event 추가
  - 모든 플랫폼 backend trait 구현 업데이트
- 장점:
  - 이론적으로 가장 “정식 추상화”에 가까움
- 단점:
  - 현재 `blinc_platform`은 window/input/event-loop 중심이며, 센서는 책임 범위가 다름
  - desktop/harmony/fuchsia까지 trait 영향 확산
  - breaking surface가 커져 도입 비용이 큼

### Option C: Blinc 내 공용 패키지 신설 + extensions는 provider 역할 (권장)

- 방식:
  - 새 공용 crate: `crates/blinc_sensors`
  - typed API, frame schema, sampling policy, buffering, error를 이 crate에 집중
  - 플랫폼별 실제 수집은 `extensions/blinc_platform_android/ios`와 native 코드가 담당
  - Rust 쪽은 `native_call("sensor", ...)`를 캡슐화한 backend adapter를 사용
- 장점:
  - 사용자에게 통일된 API 제공 가능
  - 플랫폼별 구현 차이는 provider 경계 안에 격리
  - 테스트 가능성 높음(native handler mock으로 integration test 가능)
  - `blinc_platform` 공용 trait을 건드리지 않아 리스크 낮음
- 단점:
  - crate 1개 추가 및 IDL/브리지 동기화 관리 필요

## Decision

**Option C 채택**: Blinc 내부 공용 패키지 신설 + extension provider 분리.

즉, “센서 기능은 Blinc 코어 도메인으로 관리하되, 실제 센서 획득은 플랫폼 확장에서 제공” 구조로 간다.

## Why This Fits Current Architecture

1. `blinc_platform`은 현재 window/input/lifecycle 중심이며 센서까지 흡수하면 범위가 비대해진다.
2. `blinc_core::native_bridge`가 이미 namespace 기반 확장을 지원하므로, 센서를 독립 namespace로 붙이기 좋다.
3. `blinc_app`은 Android/iOS 각각에서 platform bridge 초기화를 담당하고 있어 센서 provider 시작/중지는 여기에 연동 가능하다.
4. 통일 API 요구사항은 “공용 crate”가 가장 직접적으로 충족한다.

## Proposed Module Boundary

### New crate: `crates/blinc_sensors`

- `api`: `SensorClient`, `SensorConfig`, `SessionMode`
- `types`: `SensorFrame`, `SensorKind`, `Accuracy`, `TimeBase`
- `policy`: sampling profile / battery downgrade rules
- `buffer`: frame queue + backpressure policy
- `error`: typed error
- `backend`:
  - `NativeBridgeBackend` (default)
  - `MockBackend` (test)

### Existing platform extensions

- `extensions/blinc_platform_android`: SensorManager + FusedLocation provider, `sensor.*` handler 제공
- `extensions/blinc_platform_ios`: CoreMotion + CoreLocation provider, `sensor.*` handler 제공

### Bridge contract

- `docs/native_bridge/blinc_native.idl`에 `sensor` namespace 추가
- 예: `configure/start/stop/status/drain_frames/clear_buffer`

## Rollout Plan

1. `crates/blinc_sensors` scaffold + typed API 정의
2. IDL에 `sensor` namespace 추가
3. Android/iOS native bridge template에 `sensor` handler 추가
4. `blinc_app`에서 lifecycle hook(포그라운드/백그라운드)과 sensor session 제어 연결
5. integration test: mock backend + replay frame 테스트

## Noted Gap (Current Repository)

현재 일부 스캐폴딩 경로에서 native bridge 파일이 기본 생성물에 항상 포함되지 않는다.

- 예: `crates/blinc_cli/src/project.rs`의 Rust 모바일 템플릿은 `BlincNativeBridge.kt/.swift`를 직접 생성하지 않는다.
- 반면 별도 template 자산에는 bridge 샘플이 존재한다.

따라서 센서 도입 시, 템플릿/CLI 경로 정합성까지 함께 맞춰야 실제 사용자 프로젝트에서 일관된 동작을 보장할 수 있다.
