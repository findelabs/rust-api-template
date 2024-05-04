use axum::{
    routing::{get, post},
    Router,
    middleware,
    extract::Extension
};
use chrono::Local;
use clap::Parser;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::future::ready;
use std::io::Write;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tokio::net::TcpListener;
use tower::ServiceBuilder;

mod error;
mod handlers;
mod https;
mod metrics;
mod state;

use crate::metrics::{setup_metrics_recorder, track_metrics};
use handlers::{echo, handler_404, health, help, root};
use state::State;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Port to listen on
   #[arg(short, long, default_value_t = 8080, env = "API_PORT")]
   port: u16,

   /// Default global timeout
   #[arg(short, long, default_value_t = 60, env = "API_TIMEOUT")]
   timeout: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args = Args::parse();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"log\": {}}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    // Create state for axum
    let state = State::new(args.clone()).await?;

    // Create prometheus handle
    let recorder_handle = setup_metrics_recorder();

    // These should be authenticated
    let base = Router::new()
        .route("/", get(root));

    // These should NOT be authenticated
    let standard = Router::new()
        .route("/health", get(health))
        .route("/echo", post(echo))
        .route("/help", get(help))
        .route("/metrics", get(move || ready(recorder_handle.render())));

    let app = Router::new()
        .merge(base)
        .merge(standard)
		.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        )
        .route_layer(middleware::from_fn(track_metrics))
        .fallback(handler_404)
        .layer(Extension(state));

//    let addr = SocketAddr::from(([0, 0, 0, 0], args.port as u16));
//    log::info!("Listening on {}", addr);
//    axum::Server::bind(&addr)
//        .serve(app.into_make_service())
//        .await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = TcpListener::bind(addr).await.unwrap();

    log::info!("Listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
