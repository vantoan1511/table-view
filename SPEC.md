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

### Backend Extension (Go)
The `db-bridge` extension handles all direct database interactions. It is written in Go to ensure high performance, a small footprint, and zero-dependency distribution.

- **WebSocket Bridge**: Custom implementation in Go to communicate with NeutralinoJS using the defined extension protocol.
- **Database Drivers**:
  - **PostgreSQL**: Using `github.com/jackc/pgx/v5`. Configured with `simple_protocol` for robust schema alteration handling.
  - **MySQL**: Using `github.com/go-sql-driver/mysql`.
  - **SQLite**: Using `modernc.org/sqlite` (Pure Go implementation) for seamless cross-compilation without CGO.
- **Protocol**:
  - **getSchema**: Returns tables, views, and functions nested under a `schema` key.
  - **fetchTableData**: Returns flattened `rows` and `fields` for the data grid.
  - **executeQuery**: Executes raw SQL and returns results.
  - **updateCell/insertRow/deleteRows**: Core CRUD operations.

## Build & Deployment
...
- **Bundling**: `neutralino.config.json` is configured to exclude all source code, `node_modules`, and intermediate build files, including only the production frontend assets and the compiled extension binary.

## Auto-Updater Mechanism
The application features a hybrid auto-update system:
- **Frontend (.neu)**: Leverages `Neutralino.updater` API to fetch a remote `manifest.json` and replace the local `resources.neu` file.
- **Backend Extension (Go)**: Uses a custom "Fresh Updater" strategy. The Go extension can self-update by downloading a new binary, renaming its current executing process to `.old` (to bypass OS file locks), and saving the new version.
- **Cleanup**: On every startup, the frontend store automatically deletes any leftover `.old` extension binaries.
