# Gemini CLI Mandates - Table View Project

**Your code will be reviewed by Codex and Claude Code**

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

## Coding Conventions

- **Branches** always checkout to new branch before starting new feature or bug fix
- **Use arrow function** syntax for all functions.
- **Imports order**: components, composables, Pinia stores, Pinia actions, types, external libraries, other imports, each group separated by a blank line.
- **Component reuse**: Always reference to existing component at `/components` as the base for creating new components. Do not reinvent the wheel. Try to reuse when possible.
- **Single responsibility**: Each component should have a single responsibility. Do not create monolithic components.
- **Props and emits**: Components should use props to receive data from the parent component and emit events to communicate with the parent component.
- **Testability**: Components should be testable. Do not create components that are hard to test.
- **Performance**: Components should be performant. Do not create components that are slow to render.
- **Confirmation Dialog**: Use the existing `ConfirmDialog` component for all confirmation dialogs. Never use browser alerts or native confirmation dialogs.

## Codebase Architecture & Directory Map

To help future agent workflows, here is the complete structural layout of the Table View project:

```text
table-view/
├── extensions/                  # Desktop application extension backends
│   └── db-bridge/               # Rust-based database bridge (Neutralino extension)
│       ├── Cargo.toml           # Rust package configuration
│       └── src/
│           ├── main.rs          # Extension entry point, command dispatch, and event loop
│           ├── bridge.rs        # Neutralino bridge communication interface
│           ├── handler.rs       # Client request / database query execution handler
│           ├── pool.rs          # Database connection pool manager
│           └── drivers/         # Database-specific connection & query drivers
│               ├── mod.rs       # Driver manager and common traits
│               ├── postgres.rs  # PostgreSQL driver (sqlx)
│               ├── mysql.rs     # MySQL driver (sqlx)
│               ├── sqlite.rs    # SQLite driver (rusqlite)
│               ├── oracle.rs    # Oracle driver (oracle-rs / deadpool-oracle)
│               └── utils.rs     # Shared database utility functions
│
├── src/                         # Vue 3 Frontend source code
│   ├── main.ts                  # Application entry point
│   ├── App.vue                  # Root Vue component
│   │
│   ├── components/              # Vue components, namespaces:
│   │   ├── grid/                # Data table, Grid cell/header, grid toolbar, pagination
│   │   ├── layout/              # MinimizedDock, PanelHeader, TitleBar, StatusBar, WorkspaceContainer
│   │   ├── modals/              # NewConnectionModal and related connection modals
│   │   ├── panels/              # Properties, indexes, outputs, history, and ValueViewer
│   │   ├── sidebar/             # SidebarDialogs, DatabaseTree, DatabaseNode, ContextMenus (Schema, Table, etc.)
│   │   ├── sql/                 # SqlEditor, ResultsGrid (monaco or textarea wrappers)
│   │   └── ui/                  # UI atoms/dialogs (Button, Dropdown, Checkbox, Dialogs: AlterTable, CreateTable, Toast, Updater)
│   │
│   ├── stores/                  # Pinia stores for global application state
│   │   ├── about.ts             # App information & version metadata
│   │   ├── connections.ts       # Active and configured database connections state
│   │   ├── error.ts             # Global application error tracing state
│   │   ├── grid.ts              # DataGrid rendering, sorting, pagination & active row/cell state
│   │   ├── layout.ts            # Sidebar, panels, active tab, and modal visibility state
│   │   ├── schema.ts            # Selected database, schema, table metadata, table columns state
│   │   ├── tabs.ts              # Active workspace query/table tabs and history state
│   │   ├── toast.ts             # Transient UI notifications/toast message state
│   │   └── updater.ts           # Auto-update status, release manifests and changelog state
│   │
│   ├── composables/             # Custom Vue composables (useSqlEditor, useGridResizing, useKeyboardShortcuts, useTabSync, etc.)
│   ├── services/                # Communications (bridge.ts for extension WebSocket, native.ts for Neutralino API)
│   ├── router/                  # Vue router configuration (index.ts)
│   ├── types/                   # TypeScript interfaces and global schemas (index.ts)
│   ├── utils/                   # Frontend helpers (crypto.ts, logger.ts)
│   └── views/                   # Vue route views (MainView.vue)
│
├── public/                      # Static resources (icons, assets)
├── scripts/                     # Local builds and automated update helper scripts
├── package.json                 # Node project definition
├── tsconfig.json                # TypeScript compilation config
├── vite.config.ts               # Vite configuration
└── neutralino.config.json       # NeutralinoJS desktop wrapper and extension mappings
```
