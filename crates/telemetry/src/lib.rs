use opentelemetry::{propagation::TextMapPropagator, trace::TracerProvider as _};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::ExporterBuildError;
use opentelemetry_sdk::{
	Resource,
	error::OTelSdkResult,
	logs::{
		SdkLoggerProvider,
		log_processor_with_async_runtime::BatchLogProcessor as BatchLogProcessorWithRuntime,
	},
	propagation::TraceContextPropagator,
	runtime,
	trace::{
		SdkTracerProvider,
		span_processor_with_async_runtime::BatchSpanProcessor as BatchSpanProcessorWithRuntime,
	},
};
use std::{collections::HashMap, env};
use tracing::Span;
use tracing_opentelemetry::{OpenTelemetrySpanExt as _, SetParentError};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

pub struct Telemetry {
	tracer_provider: SdkTracerProvider,
	logger_provider: SdkLoggerProvider,
}

pub fn init(service_name: &'static str) -> Result<Telemetry, ExporterBuildError> {
	let resource = Resource::builder().with_service_name(service_name).build();

	let span_exporter = opentelemetry_otlp::SpanExporter::builder().with_http().build()?;
	let log_exporter = opentelemetry_otlp::LogExporter::builder().with_http().build()?;

	let span_processor =
		BatchSpanProcessorWithRuntime::builder(span_exporter, runtime::Tokio).build();
	let log_processor = BatchLogProcessorWithRuntime::builder(log_exporter, runtime::Tokio).build();

	let tracer_provider = SdkTracerProvider::builder()
		.with_resource(resource.clone())
		.with_span_processor(span_processor)
		.build();
	let logger_provider = SdkLoggerProvider::builder()
		.with_resource(resource)
		.with_log_processor(log_processor)
		.build();

	let tracer = tracer_provider.tracer(service_name);
	tracing_subscriber::registry()
		.with(EnvFilter::from_default_env())
		.with(tracing_opentelemetry::layer().with_tracer(tracer))
		.with(OpenTelemetryTracingBridge::new(&logger_provider))
		.init();

	Ok(Telemetry { tracer_provider, logger_provider })
}

pub fn current_trace_context() -> HashMap<String, String> {
	let mut carrier = HashMap::new();
	let context = Span::current().context();
	TraceContextPropagator::new().inject_context(&context, &mut carrier);
	carrier
}

pub fn set_parent_from_env(span: &Span) -> Result<(), SetParentError> {
	let carrier = env::vars().collect::<HashMap<_, _>>();
	let context = TraceContextPropagator::new().extract(&carrier);
	span.set_parent(context)
}

impl Telemetry {
	pub fn shutdown(self) -> OTelSdkResult {
		let Self { tracer_provider, logger_provider } = self;
		tracer_provider.shutdown()?;
		logger_provider.shutdown()?;
		Ok(())
	}
}
