#![allow(dead_code, unused_imports)]

mod app {
    pub use super::SelectedTraceContext;
}

#[path = "../src/panels/mod.rs"]
mod panels;
#[path = "../src/theme.rs"]
mod theme;

include!("../src/app.rs");

#[test]
fn app_source_can_be_included_in_integration_tests() {
    assert_eq!(MAX_EXPORT_STREAM_PAYLOAD_BYTES, MAX_NETWORK_PAYLOAD_BYTES);
}
