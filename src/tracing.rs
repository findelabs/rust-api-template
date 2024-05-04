use init_tracing_opentelemetry::{
    init_propagator, //stdio,
    otlp,
    resource::DetectResource,
};
use opentelemetry::trace::TraceError;
use opentelemetry_sdk::trace::Tracer;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry::LookupSpan, Layer};

use crate::error::Error as RestError;

#[cfg(not(feature = "logfmt"))]
#[must_use]
pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    use tracing_subscriber::fmt::format::FmtSpan;
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_line_number(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    }
}

#[cfg(feature = "logfmt")]
#[must_use]
pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    Box::new(tracing_logfmt_otel::layer())
}

#[must_use]
pub fn build_loglevel_filter_layer() -> tracing_subscriber::filter::EnvFilter {
    // filter what is output on log (fmt)
    // std::env::set_var("RUST_LOG", "warn,otel::tracing=info,otel=debug");
    let otel_logging = std::env::var("OTEL_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    std::env::set_var(
        "RUST_LOG",
        format!(
            // `otel::tracing` should be a level info to emit opentelemetry trace & span
            // `otel::setup` set to debug to log detected resources, configuration read and infered
            "{},otel::tracing=trace,otel=debug,registry::handlers={},reqwest_tracing::reqwest_otel_span_builder={},registry::cache={},registry::versions={}",
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            otel_logging,
            otel_logging,
            otel_logging,
            otel_logging
        ),
    );
    EnvFilter::from_default_env()
}

pub fn build_otel_layer<S>() -> Result<OpenTelemetryLayer<S, Tracer>, TraceError>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let otel_rsrc = DetectResource::default()
        .with_fallback_service_name("cloudtechnologies.registry.api")
        .with_fallback_service_version(clap::crate_version!())
        .build();
    let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity)?;
    // to not send trace somewhere, but continue to create and propagate,...
    // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
    // or to `std::io::stdout()` to print
    //
    // let otel_tracer = stdio::init_tracer(
    //     otel_rsrc,
    //     stdio::identity::<stdio::WriteNoWhere>,
    //     stdio::WriteNoWhere::default(),
    // )?;
    init_propagator()?;
    Ok(tracing_opentelemetry::layer()
        .with_error_records_to_exceptions(true)
        .with_tracer(otel_tracer))
}

pub fn init_subscribers() -> Result<(), RestError> {
    //setup a temporary subscriber to log output during setup
    // let subscriber = tracing_subscriber::registry()
    //     .with(build_loglevel_filter_layer())
    //     .with(build_logger_text());
    // let _guard = tracing::subscriber::set_default(subscriber);
    log::info!("Initializing tracing");

    let subscriber = tracing_subscriber::registry()
        .with(build_otel_layer()?)
        .with(build_loglevel_filter_layer())
        .with(build_logger_text());
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
