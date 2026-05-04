# Table View - Technical Specification

## Overview
Table View is a desktop-based database management system designed for speed, portability, and a modern user experience. It leverages NeutralinoJS for the desktop runtime and Vue 3 for the frontend interface.

## Architecture

### Frontend (Vue 3 + Pinia + Tailwind CSS 4)
- **Framework**: Vue 3 (Composition API)
- **State Management**: Pinia
- **Styling**: Tailwind CSS 4 (Vanilla CSS approach)
- **Icons**: Lucide Vue Next
- **Editor**: CodeMirror 6 for SQL editing
- **Communication**: Interacts with the backend extension via NeutralinoJS's WebSocket-based extension API.

### Backend Extension (Rust)
The `db-bridge` extension handles all direct database interactions. It is written in Rust to ensure high performance, a small memory footprint, and high-precision execution metrics.

- **WebSocket Bridge**: Custom implementation in Rust to communicate with NeutralinoJS using the defined extension protocol.
- **Database Drivers**: Powered by `sqlx` for asynchronous, type-safe database access.
  - **PostgreSQL**: Using `sqlx::Postgres`.
  - **MySQL**: Using `sqlx::MySql`.
  - **SQLite**: Using `sqlx::Sqlite`.
  - **Oracle**: Using `oracle-rs` with `deadpool-oracle` (Pure Rust Thin Driver + pooled connections).
- **Protocol**:
  - All response payloads include an `executionTime` (u64) field representing the time spent in the database driver (in milliseconds).
  - **getSchema**: Returns tables, views, and functions nested under a `schema` key. Oracle requests may also target a selected schema owner so the backend returns the full schema list but only the selected owner's objects.
  - **fetchTableData**: Returns flattened `rows`, `fields`, and `executionTime` for the data grid.
  - **executeQuery**: Executes raw SQL and returns `rows`, `fields`, and `executionTime`.
  - **updateCell/insertRow/deleteRows**: Core CRUD operations.

## Build & Deployment
- **Bundling**: `neutralino.config.json` is configured to exclude all source code, `node_modules`, and intermediate build files, including only the production frontend assets and the compiled extension binary.
- **Temporary Oracle Patch**: The repo currently vendors `extensions/db-bridge/vendor/oracle-rs` via Cargo patch override to support the Oracle privileged-auth flow needed by `SYSDBA` / `SYSOPER`. The long-term plan is to move the extension source and this patch into a separate repository so the app repo keeps only the shipped binary.
- **Environment Management**: Utilizes `/devmode` and `/commit` workflows to toggle between development (Inspector enabled, open security) and production (Inspector disabled, one-time token security) states.

## Auto-Updater Mechanism
The application features a hybrid auto-update system:
- **Frontend (.neu)**: Leverages `Neutralino.updater` API to fetch a remote `manifest.json` and replace the local `resources.neu` file.
- **Backend Extension (Rust)**: Uses a custom "Fresh Updater" strategy. The Rust extension can self-update by downloading a new binary, renaming its current executing process to `.old` (to bypass OS file locks), and saving the new version.
- **Cleanup**: On every startup, the frontend store automatically deletes any leftover `.old` extension binaries.
