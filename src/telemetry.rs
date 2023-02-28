use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, selectors};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::trace::TraceError;
use opentelemetry::metrics::MetricsError;

use opentelemetry_otlp::WithExportConfig;


pub fn init_metrics() -> Result<controllers::BasicController, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            aggregation::cumulative_temporality_selector(),
            opentelemetry::runtime::Tokio,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter().tonic().with_env(),
        )
        .build()
}


pub fn init_tracer() -> Result<Tracer, TraceError> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .install_batch(opentelemetry::runtime::Tokio);

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    tracer
}


pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}
