# Project Progress - Table View

## Current Status: [STABLE]
All core database management features are functional. The application has a robust Go-based backend and a fully automated CI/CD release pipeline for Windows.

## Milestones Reached

### Phase 1: Core Migration to Go
- [x] Implement WebSocket Bridge in Go.
- [x] Port PostgreSQL driver (pgx).
- [x] Port MySQL driver (mysql).
- [x] Port SQLite driver (modernc.org/sqlite).
- [x] Configure simple protocol for PostgreSQL to handle `ALTER TABLE` operations.

### Phase 2: Bug Fixes & Reliability
- [x] Fix `stdin` decoding hangs in extension.
- [x] Resolve Windows path compatibility in Neutralino config.
- [x] Align JSON payload structures between Go and Vue.
- [x] Add file-based logging for the extension.

### Phase 3: Frontend Refinement
- [x] Resolve Lucide icon resolution warnings.
- [x] Implement defensive checks in Pinia stores for schema loading.
- [x] Organize project assets: Move `mockups/` to `screenshots/` and update README.

### Phase 4: Build & Lifecycle
- [x] Configure robust exclusion patterns in `neutralino.config.json` for production bundling.
- [x] Implement hybrid auto-update mechanism for `.neu` and Go extension.
- [x] Create `manifest.json` and configure remote update URL on GitHub.

### Phase 5: DevOps & Automation
- [x] Setup GitHub Actions for automated building and releasing.
- [x] Implement automated `manifest.json` updates in the CI pipeline.
- [x] Configure automated release asset zipping (`neu build --release`).

## Upcoming Tasks

### 🛠️ Immediate Improvements
- [ ] Implement focus management for search inputs (UX polish).
- [ ] Add loading skeletons for data grid transitions.
- [ ] Implement tab-specific connection state (prevent cross-tab connection leaks).

### 🚀 New Features
- [ ] **Data Export/Import**: Support for CSV and JSON formats.
- [ ] **Query History**: Persistent storage of executed SQL queries.
- [ ] **Visual Explain**: Support for `EXPLAIN ANALYZE` visualization for Postgres/MySQL.
- [ ] **Backup/Restore**: SQL dump generation and execution.
- [ ] **Cross-Platform CI**: Add Linux and macOS to the GitHub Actions matrix.

### 🎨 Design Polish
- [ ] Add micro-animations for grid row hover and cell updates.
- [ ] Implement "Glassmorphism" effect for modal dialogs.
- [ ] Add more detailed tooltips for schema tree elements.
