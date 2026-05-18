mod bridge;
mod drivers;
mod handler;
mod pool;

use bridge::{AuthInfo, Bridge};
use drivers::Config;
use handler::handle_message;
use pool::Pool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // 1. Process command line arguments for standalone mode
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--standalone" || arg == "-s" || arg == "--help" || arg == "-h") {
        run_standalone(args).await;
        return;
    }

    let log_path = if let Some(proj_dirs) = directories::ProjectDirs::from("com", "vantoan1511", "tableview") {
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

async fn run_standalone(args: Vec<String>) {
    let mut config_path: Option<String> = None;
    let mut config_json: Option<String> = None;
    let mut test_connection = false;
    let mut query: Option<String> = None;
    let mut get_schema = false;
    let mut all_databases = false;
    let mut schema_name: Option<String> = None;
    let mut fetch_table: Option<String> = None;
    let mut limit = 100i64;
    let mut offset = 0i64;
    let mut show_help = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--standalone" | "-s" => { i += 1; }
            "--config" if i + 1 < args.len() => { config_path = Some(args[i + 1].clone()); i += 2; }
            "--config-json" if i + 1 < args.len() => { config_json = Some(args[i + 1].clone()); i += 2; }
            "--test-connection" => { test_connection = true; i += 1; }
            "--query" if i + 1 < args.len() => { query = Some(args[i + 1].clone()); i += 2; }
            "--schema" => { get_schema = true; i += 1; }
            "--all-databases" => { all_databases = true; i += 1; }
            "--schema-name" if i + 1 < args.len() => { schema_name = Some(args[i + 1].clone()); i += 2; }
            "--fetch-table-data" if i + 1 < args.len() => { fetch_table = Some(args[i + 1].clone()); i += 2; }
            "--limit" if i + 1 < args.len() => {
                if let Ok(l) = args[i + 1].parse::<i64>() { limit = l; }
                i += 2;
            }
            "--offset" if i + 1 < args.len() => {
                if let Ok(o) = args[i + 1].parse::<i64>() { offset = o; }
                i += 2;
            }
            "--help" | "-h" => { show_help = true; i += 1; }
            _ => { i += 1; }
        }
    }

    if show_help {
        print_standalone_help();
        return;
    }

    // Load connection config
    let config: Config = match (config_path, config_json) {
        (Some(path), _) => {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Config>(&content) {
                        Ok(cfg) => cfg,
                        Err(e) => {
                            eprintln!("Error: Failed to parse config file JSON: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: Failed to read config file {}: {}", path, e);
                    std::process::exit(1);
                }
            }
        }
        (_, Some(json_str)) => {
            match serde_json::from_str::<Config>(&json_str) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Error: Failed to parse config JSON string: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Error: Connection configuration required. Provide either --config <path> or --config-json <json>.");
            print_standalone_help();
            std::process::exit(1);
        }
    };

    // Create the driver
    let mut driver = match drivers::create_driver(&config.db_type) {
        Some(d) => d,
        None => {
            eprintln!("Error: Unsupported database driver type: {}", config.db_type);
            std::process::exit(1);
        }
    };

    // Establish connection
    eprintln!("Connecting to {} database...", config.db_type);
    if let Err(e) = driver.connect(&config).await {
        eprintln!("Error: Failed to connect to database: {}", e);
        std::process::exit(1);
    }
    eprintln!("Connected successfully.");

    // Perform requested action
    if test_connection {
        println!("{{\"success\":true}}");
    } else if let Some(sql) = query {
        eprintln!("Executing query: {}", sql);
        match driver.query(&sql).await {
            Ok(result) => {
                match serde_json::to_string_pretty(&result) {
                    Ok(json_output) => println!("{}", json_output),
                    Err(e) => eprintln!("Error: Failed to format result as JSON: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Error: Query failed: {}", e);
                std::process::exit(1);
            }
        }
    } else if get_schema {
        eprintln!("Retrieving database schema...");
        match driver.get_schema(all_databases, schema_name.as_deref()).await {
            Ok(schema) => {
                match serde_json::to_string_pretty(&schema) {
                    Ok(json_output) => println!("{}", json_output),
                    Err(e) => eprintln!("Error: Failed to format schema as JSON: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to retrieve schema: {}", e);
                std::process::exit(1);
            }
        }
    } else if let Some(table_name) = fetch_table {
        eprintln!("Fetching table data for {} (limit: {}, offset: {})...", table_name, limit, offset);
        match driver.fetch_table_data(&table_name, limit, offset, "", "", "").await {
            Ok(data) => {
                match serde_json::to_string_pretty(&data) {
                    Ok(json_output) => println!("{}", json_output),
                    Err(e) => eprintln!("Error: Failed to format table data as JSON: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to fetch table data: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("{{\"success\":true,\"message\":\"Connection verified. No action specified (--test-connection, --query, --schema, --fetch-table-data).\"}}");
    }

    let _ = driver.disconnect().await;
}

fn print_standalone_help() {
    println!(r#"
Table View - Database Bridge (Standalone Mode)

Usage:
  db-bridge.exe --standalone [OPTIONS]

Options:
  --standalone, -s               Explicitly run in standalone CLI mode
  --config <PATH>                Path to a JSON file containing database connection configuration
  --config-json <JSON>           JSON string containing database connection configuration
  --test-connection              Test connection to the database and exit
  --query <SQL>                  Run a SQL query and print the result as JSON
  --schema                       Get the database schema (tables, views, etc.) and print as JSON
  --all-databases                Retrieve schemas across all databases (if supported by driver)
  --schema-name <NAME>           Filter schema by a specific schema/owner name
  --fetch-table-data <TABLE>     Fetch rows from a table and print as JSON
  --limit <LIMIT>                Limit for fetching table data [default: 100]
  --offset <OFFSET>              Offset for fetching table data [default: 0]
  --help, -h                     Show this help message

Examples:
  # Test connection using a config file
  db-bridge.exe -s --config my-connection.json --test-connection

  # Retrieve schema
  db-bridge.exe -s --config my-connection.json --schema

  # Query data from a SQLite database with inline JSON config
  db-bridge.exe -s --config-json "{{\"type\":\"sqlite\",\"database\":\"test.db\"}}" --query "SELECT * FROM users"
"#);
}
