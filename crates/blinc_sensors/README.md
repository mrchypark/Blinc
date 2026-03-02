# blinc_sensors

Unified sensor API for Blinc mobile platforms.

## Scope

- Typed Rust API for sensor sessions
- Shared sensor frame/types schema
- Backend abstraction for platform-specific collection
- Native bridge backend (`sensor.*` namespace)
- Runtime query for available sensors (`supported_kinds`)

## Quick Example

```rust,ignore
use blinc_sensors::{SensorClient, SensorConfig};
use blinc_sensors::native_bridge::NativeBridgeBackend;

let client = SensorClient::new(NativeBridgeBackend);
client.configure(&SensorConfig::default())?;
let supported = client.supported_kinds()?;
client.start_session("run-001")?;
let frames = client.drain_frames(64)?;
client.stop_session("run-001")?;
```

## Native Contract

Expected native handlers:

- `sensor.configure(config_json: String) -> Bool`
- `sensor.start(session_id: String) -> Bool`
- `sensor.stop(session_id: String) -> Bool`
- `sensor.status() -> String` (JSON)
- `sensor.drain_frames(max_frames: Int32) -> String` (JSON array)
- `sensor.supported_kinds() -> String` (JSON array)
