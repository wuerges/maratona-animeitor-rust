//! Bounded-cardinality resource metrics. Guards account for cancellation and errors.
pub struct Active {
    gauge: metrics::Gauge,
    duration: metrics::Histogram,
    start: std::time::Instant,
}
impl Active {
    pub fn new(kind: &'static str) -> Self {
        let gauge = metrics::gauge!("server_resources_active", "kind" => kind);
        gauge.increment(1.0);
        Self {
            gauge,
            duration: metrics::histogram!("server_resource_lifetime_seconds", "kind" => kind),
            start: std::time::Instant::now(),
        }
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.gauge.decrement(1.0);
        self.duration.record(self.start.elapsed().as_secs_f64());
    }
}
pub fn event(kind: &'static str) {
    metrics::counter!("server_resource_events_total", "kind" => kind).increment(1);
}
