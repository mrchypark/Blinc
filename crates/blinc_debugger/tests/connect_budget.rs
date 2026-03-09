#![allow(dead_code, unused_imports)]

#[path = "../src/panels/mod.rs"]
mod panels;

#[path = "../src/theme.rs"]
mod theme;

mod app {
    include!("../src/app.rs");

    use std::collections::VecDeque;
    use std::io;
    use std::thread;
    use std::time::Duration as StdDuration;

    struct ScriptedStream {
        reads: VecDeque<io::Result<Vec<u8>>>,
        writes: Vec<Vec<u8>>,
        read_delay: StdDuration,
        write_delay: StdDuration,
    }

    impl ScriptedStream {
        fn new(reads: impl IntoIterator<Item = io::Result<Vec<u8>>>) -> Self {
            Self {
                reads: reads.into_iter().collect(),
                writes: Vec::new(),
                read_delay: StdDuration::ZERO,
                write_delay: StdDuration::ZERO,
            }
        }

        fn with_read_delay(mut self, delay: StdDuration) -> Self {
            self.read_delay = delay;
            self
        }

        fn with_write_delay(mut self, delay: StdDuration) -> Self {
            self.write_delay = delay;
            self
        }
    }

    impl io::Read for ScriptedStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let Some(result) = self.reads.pop_front() else {
                return Ok(0);
            };

            if !self.read_delay.is_zero() {
                thread::sleep(self.read_delay);
            }

            let chunk = result?;
            let len = chunk.len().min(buf.len());
            buf[..len].copy_from_slice(&chunk[..len]);
            if len < chunk.len() {
                self.reads.push_front(Ok(chunk[len..].to_vec()));
            }
            Ok(len)
        }
    }

    impl io::Write for ScriptedStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if !self.write_delay.is_zero() {
                thread::sleep(self.write_delay);
            }
            self.writes.push(buf.to_vec());
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl TimeoutStream for ScriptedStream {
        fn set_read_timeout(&self, _timeout: Option<StdDuration>) -> io::Result<()> {
            Ok(())
        }

        fn set_write_timeout(&self, _timeout: Option<StdDuration>) -> io::Result<()> {
            Ok(())
        }
    }

    fn len_prefixed_json(value: serde_json::Value) -> Vec<u8> {
        let payload = serde_json::to_vec(&value).expect("json payload should serialize");
        let mut bytes = Vec::with_capacity(4 + payload.len());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes
    }

    #[test]
    fn connect_reports_handshake_stage_in_error_message() {
        let mut stream = ScriptedStream::new([Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "hello timed out",
        ))]);

        let err =
            request_export_over_stream(&mut stream).expect_err("handshake failure should surface");

        assert!(
            err.to_string().contains("hello stage"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn connect_reports_export_stage_in_error_message() {
        let hello = len_prefixed_json(serde_json::json!({
            "type": "hello",
            "app_name": "test",
            "protocol_version": 1
        }));
        let mut stream = ScriptedStream::new([
            Ok(hello),
            Err(io::Error::new(io::ErrorKind::TimedOut, "export timed out")),
        ]);

        let err = request_export_over_stream(&mut stream)
            .expect_err("export stage failure should include stage context");

        assert!(
            err.to_string().contains("export stage"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn connect_enforces_total_one_shot_budget_across_stages() {
        let hello = len_prefixed_json(serde_json::json!({
            "type": "hello",
            "app_name": "test",
            "protocol_version": 1
        }));
        let export = len_prefixed_json(serde_json::json!({
            "type": "export",
            "export": {
                "app_name": "test",
                "protocol_version": 1,
                "events": [],
                "stats": {
                    "total_events": 0,
                    "duration_ms": 0.0,
                    "platform": "test"
                },
                "trace_entries": []
            }
        }));
        let mut stream = ScriptedStream::new([Ok(hello), Ok(export)])
            .with_read_delay(StdDuration::from_millis(120))
            .with_write_delay(StdDuration::from_millis(120));

        let err =
            request_export_over_stream_with_timeout(&mut stream, StdDuration::from_millis(350))
                .expect_err(
                    "combined hello/write/export latency should exceed the single one-shot budget",
                );

        assert!(
            format!("{err:#}").contains("one-shot timeout"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn connect_uses_single_fixed_one_shot_timeout() {
        assert_eq!(one_shot_export_timeout(), StdDuration::from_millis(350));
    }
}
