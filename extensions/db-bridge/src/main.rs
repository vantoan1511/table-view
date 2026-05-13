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
    let log_path = if let Some(proj_dirs) = directories::ProjectDirs::from("com", "vantoan1511", "table-view") {
        let data_dir = proj_dirs.data_local_dir();
        let _ = std::fs::create_dir_all(data_dir);
        data_dir.join("db-bridge.log")
    } else {
        std::path::PathBuf::from(".log")
    };

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
    
    let mut auth = AuthInfo::from_stdin();
    auth.override_from_cli();

    if auth.nl_port.is_empty() || auth.nl_token.is_empty() || auth.nl_ext_id.is_empty() {
        log::error!("Missing NeutralinoJS extension connection details.");
        std::process::exit(1);
    }

    let mut bridge = match Bridge::connect(&auth).await {
        Ok(b) => b,
        Err(e) => {
            log::error!("Failed to connect to NeutralinoJS: {}", e);
            std::process::exit(1);
        }
    };

    log::info!("Extension {} connected to NeutralinoJS.", auth.nl_ext_id);

    let pool = Arc::new(Mutex::new(Pool::new()));
    let writer_arc = bridge.writer;
    let token = bridge.token;

    loop {
        // We now pass the reader and writer separately, so reading does NOT lock writing.
        let msg = match Bridge::read_message(&mut bridge.reader, &writer_arc).await {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("WebSocket read error: {}", e);
                break;
            }
        };

        if msg.event.as_deref() == Some("windowClose") {
            log::info!("Received windowClose, shutting down.");
            let mut p = pool.lock().await;
            p.close_all().await;
            break;
        }

        let writer_clone = Arc::clone(&writer_arc);
        let pool_clone = Arc::clone(&pool);
        let token_clone = token.clone();

        tokio::spawn(async move {
            handle_message(msg, writer_clone, token_clone, pool_clone).await;
        });
    }
}
