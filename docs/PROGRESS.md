# Project Progress - Table View

## Current Status: [STABLE]
All core database management features are functional. The application has a robust Rust-based backend and a fully automated CI/CD release pipeline for Windows.

## Milestones Reached

### Phase 1: Core Migration to Rust
- [x] Implement WebSocket Bridge in Rust.
- [x] Port PostgreSQL driver (sqlx).
- [x] Port MySQL driver (sqlx).
- [x] Port SQLite driver (sqlx).
- [x] Configure simple protocol for PostgreSQL to handle `ALTER TABLE` operations.

### Phase 2: Bug Fixes & Reliability
- [x] Fix `stdin` decoding hangs in extension.
- [x] Resolve Windows path compatibility in Neutralino config.
- [x] Align JSON payload structures between Rust and Vue.
- [x] Add file-based logging for the extension.
- [x] Fix extension process termination using explicit shutdown signals and awaited frontend dispatch.
- [x] Update project mandates to correctly reflect Rust-based backend architecture.

### Phase 3: Frontend Refinement
- [x] Resolve Lucide icon resolution warnings.
- [x] Implement defensive checks in Pinia stores for schema loading.
- [x] Organize project assets: Move `mockups/` to `screenshots/` and update README.

### Phase 4: Build & Lifecycle
- [x] Configure robust exclusion patterns in `neutralino.config.json` for production bundling.
- [x] Implement hybrid auto-update mechanism for `.neu` and Rust extension.
- [x] Create `manifest.json` and configure remote update URL on GitHub.

### Phase 5: DevOps & Automation
- [x] Setup GitHub Actions for automated building and releasing.
- [x] Implement automated `manifest.json` updates in the CI pipeline.
- [x] Configure automated release asset zipping (`neu build --release`).
- [x] Implement structured `CHANGELOG.md` following Keep a Changelog.

### Phase 6: UI Functionality & Feature Integration
- [x] Implement Home navigation and "More" context menu in TitleBar.
- [x] Enable global search focus (⌘K/Ctrl+K) for quick navigation.
- [x] Refactor Sidebar with functional connection and schema switchers.
- [x] Implement schema refresh mechanism to sync with backend.
- [x] Wire GridToolbar search, column visibility, and rows-per-page controls.
- [x] Implement functional Rows Per Page dropdown in Pagination.
- [x] Connect StatusBar to dynamic row counts and update check feedback.
- [x] Add JSON-based connection profile import in NewConnectionModal.
- [x] Fix critical `loadTable` bug to allow same-table data refreshing.
- [x] Resolve TypeScript "possibly undefined" and type mismatch errors.

### Phase 7: Performance & Quality
- [x] Implement backend-level query timing for accurate performance metrics.
- [x] Integrate real execution time into the UI StatusBar and Grid stores.
- [x] Fix production build bottlenecks and TypeScript inference errors.
- [x] Standardize environment workflows (`/devmode`, `/commit`).
- [x] Automate documentation synchronization in the commit pipeline.
- [x] Automate changelog maintenance in the release workflow.
- [x] Eliminate all compilation warnings in the Rust bridge.

## Upcoming Tasks

### 🛠️ Immediate Improvements
- [x] Add loading skeletons for data grid transitions.
- [x] Implement tab-specific connection state (prevent cross-tab connection leaks) using LRU connection pool in backend.
- [x] Enhance "Export CSV" functionality with actual backend execution and native OS save dialog.

### 🚀 New Features
- [x] **Workspace Panels**: Implement real-time Output, Properties, and Indexes views.
- [x] **Display All Schemas**: Add toggle in connection settings and implement schema switching in frontend and backend.
- [x] **Oracle DB Support**: Create detailed implementation plan using pure Rust thin driver (`oracle-rs`).
- [ ] **Oracle DB Support**: Implement backend driver and frontend UI components.
- [ ] **SQL Workbench**: Full implementation of the query editor and result set viewer.
- [ ] **Schema Management**: Create/Drop schema support (backend & UI).
- [ ] **Data Import/Export**: Support for JSON and SQL dump formats.
- [ ] **Query History**: Persistent storage of executed SQL queries in the Timeline tab.
- [ ] **Visual Explain**: Support for `EXPLAIN ANALYZE` visualization for Postgres/MySQL.
- [ ] **Cross-Platform CI**: Add Linux and macOS to the GitHub Actions matrix.

### 🎨 Design Polish
- [ ] Add micro-animations for grid row hover and cell updates.
- [ ] Implement "Glassmorphism" effect for modal dialogs.
- [ ] Add more detailed tooltips for schema tree elements.
- [ ] Implement dark/light mode toggle persistence in Neutralino storage.
