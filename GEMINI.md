# Gemini CLI Mandates - Table View Project

## Core Engineering Standards

- **Backend Architecture**: All backend bridge logic must reside in the `extensions/db-bridge` directory and be written in **Rust**.
- **Database Driver Guidelines**:
  - Use `sqlx` or dedicated Rust drivers (like `rusqlite`, `tokio-postgres`, `mysql_async`) to maintain high performance and safety.
  - For PostgreSQL, ensure proper connection pooling and error handling. Always cast both operands of dynamic comparisons (e.g., column comparisons with dynamic placeholders) to the same explicit type (e.g. `col::text = $1::text` or `col::text IN ($1::text)`) to prevent prepared statement type mismatch errors (e.g. `operator does not exist: text = bigint`).
  - For Oracle, use `oracle-rs` with `deadpool-oracle` for pure Rust thin-driver connectivity without Oracle Instant Client or OCI/ODPI-C dependencies.
- **Frontend State**: Use Pinia for global state. Ensure WebSocket response handling in stores matches the expected payload structures (e.g., `payload.schema` vs direct flattening).
- **Styling**: Maintain the established dark-mode aesthetic using Tailwind CSS 4.
- **Bundling**: Always update `neutralino.config.json` exclusion patterns when adding new source files or directories to keep the production bundle lean.
- **Auto-Update**:
  - For extension updates, use the `dbBridge.updateExtension` event to trigger the custom "Fresh Updater" logic in the Rust backend.
  - For frontend updates, use the standard `Neutralino.updater.install()` method.
  - Maintain the update manifest URL in the `updater` store pointing to the `main` branch on GitHub (`https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest.json`).
  - Ensure version strings follow Semantic Versioning (SemVer).

## Workflow Mandates

- **Validation**: After modifying the Rust extension, always perform a `cargo build` to ensure the binary compiles correctly before testing with NeutralinoJS.
- **Logging**: Maintain a centralized log file in a writable system directory (e.g., `%LOCALAPPDATA%` on Windows), capturing both backend (Rust) and frontend (UI) diagnostic information for unified troubleshooting.
- **Pathing**: Use Windows-compatible backslash paths (`\\`) in `neutralino.config.json` for extension commands.

## UI/UX Standards

- **Icons**: Use Lucide Vue Next. Always import icons explicitly in the `<script setup>` block.
- **Feedback**: Use the existing `toast` and `error` stores to provide immediate user feedback for all background operations.

## Frontend Coding Conventions

- **Use arrow function** syntax for all functions.
- **Imports order**: components, composables, Pinia stores, Pinia actions, types, external libraries, other imports, each group separated by a blank line.

## Coding conventions:

- Always reference to existing component at `/components` as the base for creating new components. Do not reinvent the wheel.
