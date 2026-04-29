mod bridge;
mod drivers;
mod handler;
mod pool;

use bridge::{AuthInfo, Bridge};
use handler::handle_message;
use pool::Pool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // Initialize logging to file next to executable
    let exe_path = std::env::current_exe().unwrap_or_default();
    let log_path = exe_path.with_extension("log");
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .target(if let Some(f) = log_file {
            env_logger::Target::Pipe(Box::new(f))
        } else {
            env_logger::Target::Stderr
        })
        .init();

    log::info!("--- db-bridge-rs extension starting ---");
    log::info!(
        "Working directory: {}",
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );
    log::info!("Executable path: {}", exe_path.display());

    // ─── Read auth from stdin (JSON, single object) ─────────────────────────
    let mut auth = AuthInfo::from_stdin();
    log::info!("Received auth from stdin: {:?}", auth);

    // ─── Override with CLI flags ────────────────────────────────────────────
    auth.override_from_cli();

    if auth.nl_port.is_empty() || auth.nl_token.is_empty() || auth.nl_ext_id.is_empty() {
        log::error!("Missing NeutralinoJS extension connection details.");
        std::process::exit(1);
    }

    // ─── Connect to Neutralino WebSocket ────────────────────────────────────
    let bridge = match Bridge::connect(&auth).await {
        Ok(b) => Arc::new(Mutex::new(b)),
        Err(e) => {
            log::error!("Failed to connect to NeutralinoJS: {}", e);
            std::process::exit(1);
        }
    };

    log::info!("Extension {} connected to NeutralinoJS.", auth.nl_ext_id);

    // ─── Connection Pool ────────────────────────────────────────────────────
    let pool = Arc::new(Mutex::new(Pool::new()));

    // ─── Listen loop ────────────────────────────────────────────────────────
    loop {
        let msg = {
            let mut b = bridge.lock().await;
            match b.read_message().await {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!("WebSocket read error: {}", e);
                    break;
                }
            }
        };

        // Handle windowClose to exit gracefully
        if msg.event.as_deref() == Some("windowClose") {
            log::info!("Received windowClose, shutting down.");
            let mut p = pool.lock().await;
            p.close_all().await;
            break;
        }

        let bridge_clone = Arc::clone(&bridge);
        let pool_clone = Arc::clone(&pool);

        tokio::spawn(async move {
            handle_message(msg, bridge_clone, pool_clone).await;
        });
    }
}
