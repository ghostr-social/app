use ghostr_gateway::gateway_runtime::{report_http_server_failure, supervise_http_server};
use log::{LevelFilter, Log, Metadata, Record};
use std::io;
use std::sync::Mutex;

static LOGGER: RecordingLogger = RecordingLogger(Mutex::new(None));

struct RecordingLogger(Mutex<Option<String>>);

impl Log for RecordingLogger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        *self.0.lock().expect("log lock") = Some(record.args().to_string());
    }

    fn flush(&self) {}
}

#[tokio::test]
async fn reports_a_background_http_server_failure_without_panicking() {
    log::set_logger(&LOGGER).expect("test logger");
    log::set_max_level(LevelFilter::Warn);

    supervise_http_server(
        async { Err(io::Error::other("listener failed")) },
        report_http_server_failure,
    )
    .await;

    let message = LOGGER.0.lock().expect("log lock").clone();
    assert!(message.is_some_and(|value| value.contains("listener failed")));
}
