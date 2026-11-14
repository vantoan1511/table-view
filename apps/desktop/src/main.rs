#[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
mod config;
#[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
mod theme;
#[cfg(feature = "native-ui")]
mod window;

use std::process::ExitCode;

#[cfg(not(feature = "native-ui"))]
fn main() -> ExitCode {
    eprintln!(
        "table-view-desktop built without 'native-ui' feature enabled. Rebuild with --features native-ui."
    );
    ExitCode::from(2)
}

#[cfg(feature = "native-ui")]
fn main() -> ExitCode {
    window::run_native_window(config::AppWindowConfig::default());
    ExitCode::SUCCESS
}
