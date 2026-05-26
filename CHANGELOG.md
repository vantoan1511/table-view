# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-05-26

### Added
- **Autocomplete: Context-Aware Suggestions**: Implemented intelligent autocomplete suggestions for both the SQL Editor (fetching schema and metadata via `diagramStore`) and the Grid Filter Bar (offering column names and operators).
- **Sidebar: Collapse All Action**: Added a global "Collapse All" button to the database tree header to reset the expansion state of all connection, database, and schema nodes.
- **Database: Disconnection Feature**: Implemented a secure database disconnection flow that terminates active connection pools in the Rust backend and clean up frontend state/schema.
- **UI: Button severity styles**: Enhanced the custom `Button` component with rich structural variants (filled, outline, ghost, subtle) and severity options (primary, secondary, success, danger, warning, etc.).
- **Grid: Filter persistence**: Enhanced useTableData to preserve and restore filter and sorting states per tab when switching back and forth.

## [0.3.0] - 2026-05-25

### Added
- **Diagram: Schema Relationship Diagram (ERD)**: Implemented an interactive database schema relationship diagram (ERD) feature to visualize tables and foreign key relationships.
- **Grid: Footer Action Buttons**: Relocated row action buttons (Add Row, Delete Selected) to the pagination footer at the bottom-left of the data grid, clean-up of the grid toolbar.

### Fixed
- **UI: Sidebar Dialogs**: Fixed sidebar delete confirmation dialogs not closing correctly on cancel, and cleaned up unused components.
- **Sidebar: Connection Refresh**: Ensured the "Display All Databases" preference is correctly respected during schema and connection refresh actions.

## [0.2.15] - 2026-05-24

### Added
- **Grid: In-Memory Tab Caching**: Implemented a highly optimized in-memory table data cache per tab to eliminate redundant database queries when switching tabs, while maintaining consistency with active row edits and offering force-refresh capability.
- **Grid: Persistent Layout & Column Editing**: Implemented state persistence for grid settings per tab, and extracted a reusable, modular column editor for cleaner layout management.
- **Grid: Standardized Input Components**: Replaced manual HTML inputs with consolidated custom UI elements (e.g., standardizing on the unified `Checkbox` component).
- **Database: Robust Deletion Workflow**: Added a secure 2-step database deletion flow including background session termination and clean schema/state invalidation.

### Changed
- **Grid: Resizable and Autodistributed Columns**: Refactored grid column width calculation, rendering, and distribution using a synchronized `colgroup` layout, eliminating flickering, handling overflow via dynamic CSS constraints, and preserving tab-specific column structures.
- **Documentation**: Fully documented database tree layouts, components architecture, dynamic stores, and architectural coding conventions under `GEMINI.md`.

### Fixed
- **Database: Preferred DB Display**: Resolved issues on connection edits to ensure preferred databases render correctly.
- **Architecture: Reactivity & Hardening**: Fixed reactivity inconsistencies in grid state caching and addressed architectural violations identified in PR reviews by restoring official settings and cleanup bounds.

## [0.2.14] - 2026-05-23

### Fixed
- **Grid: Resolve Relation Not Found**: Resolved relation not found error on column sort and pagination.

## [0.2.13] - 2026-05-23

### Fixed
- **Updater: Restart Binary Mismatch**: Resolved post-update restart error by dynamically querying the running executable path using the current process ID (`window.NL_PID`) via PowerShell, ensuring it successfully kills and restarts the correct binary.

## [0.2.12] - 2026-05-23

### Added
- **UI: Dynamic Release Notes**: Implemented real-time release notes fetching from the GitHub Releases API (queried via tags `v<version>`) in the updater dialog.
- **UI: Inline Markdown Support**: Enabled rich formatting (bold, italic, inline code) for release notes using a secure HTML-escaped inline parser in the updater dialog.
- **Testing: Robust Coverage**: Added comprehensive unit tests for `updaterStore` update check and API fallback logic.
- **Backend: Driver Refactoring**: Consolidated database driver implementations (PostgreSQL, MySQL, SQLite, Oracle) by extracting duplicate connection and transaction patterns into a unified `utils.rs`.
- **UI: Centralized Input Component**: Extracted duplicate modal schemas from `CreateDatabaseDialog` and `CreateSchemaDialog` into a new, reusable `InputDialog.vue` component.

## [0.2.11] - 2026-05-18

### Added
- **Backend: Standalone CLI Mode**: Added standalone mode support (`--standalone` or `-s`) to the `db-bridge` Rust binary, allowing connection testing, querying, schema retrieval, and table data fetching directly from the CLI.

## [0.2.10] - 2026-05-18

### Added
- **Backend: Schema & Database Deletion**: Implemented `drop_schema` and `drop_database` across all drivers (PostgreSQL, MySQL, SQLite, Oracle).
- **UI: Sidebar Dialogs**: Extracted all sidebar-related confirmation and creation dialogs into a dedicated `SidebarDialogs.vue` component for better maintainability.
- **UI: Deletion Workflows**: Integrated interactive deletion for databases and schemas with confirmation dialogs.
- **UI: Data Grid & SQL Results Virtualization**: Fully implemented high-performance virtual scrolling for both main Data Grid and SQL Results Grid to handle large datasets.
- **UI: Grid Cell Value Selection**: Enabled select-text on grid cell values and added a "Copy Cell Value" option to the grid context menu.

### Changed
- **Architecture: State Cleanup**: Enhanced `schemaStore` with formal cleanup logic to prevent state leaks when deleting connections.
- **Production: Security Hardening**: Enforced `tokenSecurity: "one-time"`, `dataLocation: "system"`, and disabled developer inspector in `neutralino.config.json`.
- **UI: Database Tree**: Refactored `DatabaseTree.vue` to use the new centralized `SidebarDialogs` component.
- **UI: SQL Editor Layout Optimization**: Restructured the results header by integrating tab selection (Results/Messages) and row count metadata directly into the result pane, maximizing the active workspace.
- **UI: Button Icon Standardization**: Standardized icon button bindings across dialogs to use consistent Vue property definitions (e.g., `:icon`).

## [0.2.9] - 2026-05-16

### Added
- **UI: Create Table Dialog**: Implemented a comprehensive dialog for defining new table structures with custom `Checkbox` and `DropdownMenu` components.
- **UI: Table Management**: Added "Alter Table" and "Drop Table" actions to the table context menu.
- **Backend: Table Operations**: Extended all database drivers (PostgreSQL, MySQL, SQLite, Oracle) to support `create_table` and `drop_table` functionality.
- **Backend: Multi-Statement Support**: Updated the PostgreSQL driver to use the simple query protocol, enabling execution of semicolon-separated SQL scripts.
- **Backend: System Schema Filtering**: Improved database tree hygiene by automatically filtering out system-level schemas and databases across all drivers.

### Changed
- **Architecture: Type Safety**: Refactored the grid store to use a formal `TableColumn` interface, enforcing strict typing for table management.
- **UI: Database Tree**: Refined the sidebar hierarchy and context menu interactions for better usability.

## [0.2.7] - 2026-05-15

### Added
- **Grid: Enhanced Cell Selection**: Implemented a premium ring highlight and background tint for selected cells in `DataGrid.vue`.
- **UI: Value Viewer**: Optimized the dedicated 'Value' tab for high-convenience viewing and editing of large cell content.
- **UI: Dropdown Menu**: Introduced a reusable `DropdownMenu.vue` component for consistent UI interactions.

### Changed
- **Backend: Rich Schema Metadata**: Updated Rust drivers (PostgreSQL, MySQL, SQLite, Oracle) to return enhanced metadata for precise grid operations.
- **Architecture: Data Management**: Refactored `grid.ts` and `useTableData.ts` to use a more robust state handling and value conversion logic.
- **UI: Grid Toolbar & Pagination**: Refined toolbar and pagination components for better responsiveness and visibility control.

## [0.2.6] - 2026-05-14

### Added
- **Logging: System Data Directory**: Relocated the application log file to a writable system directory (e.g., `%LOCALAPPDATA%` on Windows) for improved reliability and compliance with OS standards.
- **Backend: Persistent Logs**: Updated the Rust `db-bridge` to automatically create and manage the log file in the system data path.

### Changed
- **Neutralino: Storage Configuration**: Updated `neutralino.config.json` to use system-level storage and data locations.


## [0.2.5] - 2026-05-13

### Added
- **UI: Hierarchical Context Menus**: Implemented dedicated context menus for Databases, Schemas, and Tables, providing specialized actions for each entity type.
- **Samples: Customer Dataset**: Added `customers.sql` script for easy database initialization and testing.
- **Patterns: Repository Skill**: Introduced project-scoped `SKILL.md` to formalize development patterns and instincts.

### Changed
- **Architecture: Sidebar Hierarchy**: Refactored `DatabaseTree.vue` to support deeper nesting and more intuitive navigation.
- **Backend: Driver Optimization**: Enhanced Rust `db-bridge` with improved connection pooling and driver-specific performance tweaks for all supported engines.
- **UI: Visual Polish**: Improved the `ColorPicker` component and refined keyboard shortcut handling for better accessibility.

## [0.2.4] - 2026-05-09

### Added
- **Quality: Code Standards**: Integrated ESLint and Prettier for automated linting and formatting across the entire TypeScript/Vue codebase.

### Changed
- **Production: Hardening**: Finalized production-ready settings in `neutralino.config.json` and `index.html`, enforcing `tokenSecurity: "one-time"` and disabling the developer inspector.
- **Maintenance: Global Formatting**: Applied project-wide code style cleanup to ensure consistency across all source files.


## [0.2.3] - 2026-05-09

### Added
- **Architecture: Modular Stores**: Refactored schema and grid stores into domain-specific actions and state modules for better maintainability.
- **UI: Database Tree Decomposition**: Decoupled the database tree into granular node components (Connection, Database, Schema) for improved responsiveness.

### Fixed
- **Database: Schema Loading**: Resolved the "infinite loading" issue in the database sidebar by correcting payload handling and synchronizing the `db-bridge` binary.
- **Security: Driver Hardening**: Hardened PostgreSQL, MySQL, and SQLite drivers against SQL injection using parameterized queries.
- **Performance: Redundant Fetching**: Optimized sidebar responsiveness by removing duplicate schema fetch calls.


## [0.2.2] - 2026-05-09

### Changed
- **Production: Hardening**: Re-enforced `tokenSecurity: "one-time"` and disabled developer inspector for official release.
- **Assets: Icon Paths**: Standardized icon paths and inclusion in the production bundle.
- **Neutralino: Globals Path**: Standardized the globals script path to use `%PUBLIC_URL%`.

### Fixed
- **CI/CD: Release Workflow**: Fixed a PowerShell syntax error in the GitHub Actions workflow that caused Windows installer builds to fail.
- **Production: Build Assets**: Added `data-skip-asset` to the Neutralino globals script to prevent Vite from attempting to bundle it during production builds.



## [0.2.0] - 2026-05-08

### Added
- **Distribution: Windows Installer**: Implemented a professional Windows installer using Inno Setup. It provides a per-user installation path, desktop shortcuts, and an integrated uninstaller.
- **CI/CD: Automated Packaging**: Updated the GitHub Actions release workflow to automatically compile and attach the `setup.exe` installer to every new release tag.

### Fixed
- **UI: All Databases Toggle**: Fixed a TypeScript compilation error in `App.vue` that prevented builds when the `displayAllDatabases` flag was enabled.


## [0.1.9] - 2026-05-08

### Added
- **Feature: All Databases Toggle**: Migrated `displayAllSchemas` to `displayAllDatabases` across the UI and backend drivers. Users can now choose to fetch all databases on a server or only the configured one for Postgres, MySQL, and SQLite.

### Fixed
- **Database: SQLite Driver**: Fixed a Rust build error in the SQLite driver caused by a missing `databases` field in the schema result payload.

### Changed
- **Production: Hardening**: Finalized production-ready settings in `neutralino.config.json`, enforcing `tokenSecurity: "one-time"` and disabling the developer inspector.

## [0.1.8] - 2026-05-07

### Added
- **UI: Connection Color Flags**: Implemented a vertical color indicator system in the sidebar and matching horizontal color strips on workspace tabs for immediate environment context (e.g., Prod vs Dev).
- **UI: Color Preview**: Added a real-time flag preview in the New Connection modal when selecting connection colors.
- **UI: Multi-Tag Support**: Connections now support multiple comma-separated tags, rendered as individual stylized badges in the sidebar.

### Changed
- **UI: Connection Modal Cleanup**: Simplified the connection setup by removing redundant fields (Environment, Timeout, Application Name, Comment) in favor of flexible tagging.
- **Production: Hardening**: Verified production settings for NeutralinoJS, including one-time token security and inspector disabled states.

## [0.1.7] - 2026-05-07

### Changed
- **UI: Centralized Connection Controls**: Moved the "New Connection" button from the TitleBar to the top of the Sidebar for better contextual relevance.
- **UI: Database Tree Cleanup**: Removed the redundant "Add Connection" button at the bottom of the database tree.
- **UI: Iconography**: Refined the Oracle database icon for better visual alignment.

### Fixed
- **SQL Editor**: Restored editor tab persistence feature. The database tree context menu no longer forces a new SQL editor tab every time, correctly prioritizing existing active editor sessions.
- **Database**: Resolved 'empty host' connection failures by URL-encoding credentials to safely handle special characters like `@` or `#` in passwords for PostgreSQL and MySQL.
- **Lifecycle**: Cleaned up obsolete auto-save and closure event handlers in the main App component.

## [0.1.6] - 2026-05-07

### Added
- **Architecture: Multi-Connection Schema Support**: Refactored `schemaStore` to support independent schema state for multiple concurrent database connections.
- **UI: Unified Database Tree**: Replaced legacy `ConnectionList` and `SchemaTree` with a consolidated `DatabaseTree.vue` hierarchical view.
- **Store: Connection-Aware Tabs**: Updated `tabsStore` to handle connection-specific table opening, preventing cross-database state leakage.
- **UI: DB-Specific Icons**: Added a new set of database icons (`DbIcon.vue`) for improved visual hierarchy.

### Changed
- **Build: Neutralino Globals Path**: Standardized the Neutralino globals script path in `index.html` to use `%PUBLIC_URL%` for production builds.

## [0.1.5] - 2026-05-06

### Changed
- **Neutralino: Core Dependency Update**: Updated `@neutralinojs/lib` and runtime binaries to v6.7.0.
- **Security: Production Hardening**: Enforced `tokenSecurity: "one-time"` and disabled the developer inspector for production builds.

## [0.1.4] - 2026-05-06

### Changed
- **Neutralino: Modernized configuration for v6**: Updated `neutralino.config.json` to the latest v6 structure and refined build exclusion patterns.

### Fixed
- **Build: Resolved path resolution error**: Fixed `ENOENT` error during `neu build --release` by correctly configuring the frontend library patch mechanism.

## [0.1.3] - 2026-05-05

### Added
- **SQL Editor: Query Persistence**: Implemented automatic state saving for SQL queries using Neutralino storage, ensuring work is never lost.
- **SQL Editor: Tab Lifecycle Management**: Closed tabs are now marked as 'closed' instead of being deleted, allowing them to be recovered later.
- **SQL Editor: Tab Selector Dialog**: Added a new dialog that appears when multiple editors exist for a connection, allowing users to choose or create new ones.
- **SQL Editor: Tab Renaming**: Enabled double-click renaming for editor tabs to improve workspace organization.
- **SQL Editor: Delete Permanently**: Added explicit deletion controls via context menu and selector dialog.

### Changed
- **Persistence: Debounced Writes**: Optimized storage performance by debouncing state writes to disk.
- **Reactivity: Editor Synchronization**: Resolved issues with SQL content not populating correctly by using unique tab keys for component lifecycle management.

### Fixed
- **Stability: Storage Key Format**: Fixed a critical issue where storage keys with dots were rejected by Neutralino, causing persistence failures.
- **Bug: Tab Content Recovery**: Fixed root cause where closing a tab deleted its content permanently.

## [0.1.2] - 2026-05-05

### Added
- **Logging: Centralized Application Logging**: Consolidated both frontend (UI) and backend (Rust) diagnostic information into a single `.log` file in the application root.
- **UI: Tab Reordering**: Implemented drag-and-drop tab reordering in the TitleBar for better workspace organization.
- **UI: Horizontal Tab Scrolling**: Added responsive horizontal scrolling to the tab strip to handle many open tabs gracefully.

### Changed
- **UI: TitleBar Cleanup**: Removed redundant Home button and "More" context menu to simplify the interface and focus on core navigation.
- **UI: Removed Split-View Drag**: Disabled the drag-to-split-view feature in favor of tab reordering and cleaner layout management.

### Fixed
- **Stability: UI Log Routing**: Fixed the lack of persistent UI logs by routing all `console` calls through the database bridge extension to the central log file.
- **Store: Fixed tabsStore syntax errors** and restored missing `restoreTab` method.

## [0.1.2] - 2026-05-05

### Fixed
- **Database: Resolved PostgreSQL UUID type mismatch error** by implementing explicit `::text` casting in the Rust backend drivers for `update` and `delete` operations.
- **Grid: Improved focus management** for a more standard user experience.
  - Inline edit mode is now automatically discarded when clicking outside the grid or inspector panels.
  - Selecting a cell now correctly clears when clicking on sidebar or status bar.
- **Architecture: Standardized modal interactions** across GlobalErrorDialog, AboutDialog, and NewConnectionModal to ensure reliable focus preservation.
- **UI: Fixed version display** in the About dialog to correctly read from app configuration.

## [0.1.1] - 2026-05-05

### Added
- **Grid: Implemented cell-level selection** with a premium ring highlight and background tint.
- **Value Viewer: Added a dedicated 'Value' tab** to the console panel for high-convenience viewing and editing of large cell content.
  - Supports monospaced typography for structured data.
  - Includes a "Copy to clipboard" shortcut.
- **Transactional Editing**: Implemented Save/Discard icon buttons for both inline grid editing and the Value panel.
- **UX: Auto-tab switching**: The application now automatically switches to the 'Value' tab when a cell is clicked, providing instant visibility.

### Fixed
- UI: Resolved multiple "possibly undefined" TypeScript warnings in `DataGrid.vue` and `WorkspaceContainer.vue`.
- Cleaned up deprecated `PanelRail.vue` component from the codebase.
- Enforced production security settings and globals path for the release build.

## [0.1.0] - 2026-05-04

### Added
- **Database: Integrated Oracle Database support** using the `oracle-rs` pure-Rust thin driver.
  - No dependency on Oracle Instant Client or OCI/ODPI-C libraries.
  - Supports Service Name, SID, and TNS connectivity modes.
  - Full support for Oracle-specific schema management and system roles (SYSDBA, SYSOPER, etc.).
- Performance: Optimized application startup with parallel initialization and lazy-loaded components.
- UI/UX: Added a premium branded splash screen with smooth cross-fade transitions.
- SQL Editor: Implemented professional dark mode and high-contrast custom syntax highlighting.
- Architecture: Introduced a reusable `useDebounce` composable for standardizing UI responsiveness.
 
 ## [0.0.25] - 2026-05-04
 
 ### Added
 - UI/UX: Implemented a premium, high-fidelity splash screen with a branded gradient logo and uppercase typography.
 - Performance: Optimized startup sequence by parallelizing Neutralino initialization with Vue mounting.
 - Performance: Implemented code splitting for secondary components (Modals, Toasts, Dialogs) to reduce initial bundle execution time.
 - Performance: Added smooth CSS cross-fade transitions between the splash screen and the application shell.
 - Architecture: Deferred non-critical background services (like update checks) to post-startup for zero-contention boot.

## [0.0.24] - 2026-05-04

### Added
- Database: Implemented synchronous connection verification for PostgreSQL and MySQL drivers to ensure credential validity during testing.
- UI/UX: Added detailed error feedback popups for failed connection tests.
- SQL Editor: Implemented professional dark mode and high-contrast custom syntax highlighting for improved readability.
- Performance: Integrated loading animations and execution guards for the SQL query runner.
- Architecture: Introduced a reusable and highly configurable `useDebounce` composable for standardizing UI performance.

### Fixed
- Connection: Resolved "Test Connection" bug where it would incorrectly report success before verifying the handshake.
- UI/UX: Improved clarity in the connection modal by changing the "Ready" state indicator to a neutral gray.

## [0.0.23] - 2026-04-29

### Fixed
- CI/CD: Resolved "unstaged changes" error during branch switching in GitHub Actions by resetting build-time file modifications.

## [0.0.22] - 2026-04-29

### Fixed
- CI/CD: Resolved `manifest.json` update failure in GitHub Actions by correctly sequencing branch checkouts.

### Changed
- CI/CD: Parallelized backend and frontend build steps to reduce total release time.
- Performance: Optimized Rust release profile for faster CI builds while maintaining stripped binaries.

## [0.0.21] - 2026-04-29

### Fixed
- Reliability: Resolved `db-bridge.exe` process termination issues using explicit shutdown signals and awaited frontend dispatch.

### Changed
- Mandates: Updated project architecture documentation (`GEMINI.md`) to correctly reflect the Rust-based backend.

## [0.0.20] - 2026-04-29

### Added
- CI/CD: Migrated GitHub Actions release workflow to Rust architecture with optimized caching.
- Automation: Implemented automated documentation synchronization for README, PROGRESS, and SPEC files.
- Workflow: Enhanced release process with mandatory changelog maintenance.

### Added
- Performance: High-precision backend execution timing (ms) reported directly from Rust drivers.
- DevOps: New `/sync-docs` workflow to automate synchronization of `PROGRESS.md`, `SPEC.md`, and `README.md`.
- Workflow: Integrated documentation synchronization into the `/commit` pipeline.

### Fixed
- Build: Resolved 11 TypeScript inference errors in `App.vue` that were blocking production builds.
- Bundling: Fixed `INEFFECTIVE_DYNAMIC_IMPORT` warnings and resolved script resolution issues in `index.html`.

## [0.0.18] - 2026-04-29

### Added
- Architecture: **Migrated backend extension from Go to Rust** for improved performance and safety.
- Features: Implemented tab-specific connection state using an LRU connection pool.
- Workflow: Added `/devmode` workflow for quick toggling of development settings.

### Fixed
- Stability: Resolved bridge deadlocks and improved error handling in the Rust extension.
- Compatibility: Standardized JSON payload structures for Rust-Vue interop.

## [0.0.17] - 2026-04-27

### Fixed
- UI: Restored index.html global script placeholder for correct production builds.

## [0.0.16] - 2026-04-27

### Fixed
- Auto-Updater: Batch swapper now forcefully kills `db-bridge.exe` before attempting to replace the binary, resolving infinite "Access is denied" loop on Windows.

## [0.0.15] - 2026-04-27

### Changed
- Architecture: Refactored the auto-update system to be Core-Led. The frontend now orchestrates downloads via native OS commands (PowerShell), bypassing CORS and keeping the extension clean of update logic.

## [0.0.14] - 2026-04-27

### Fixed
- Auto-Updater: Completely redesigned the update process to use a native atomic swapper (Windows Batch). This resolves CORS errors and "Access Denied" file lock issues by performing the swap after the application exits.

## [0.0.13] - 2026-04-27

### Fixed
- Auto-Updater: Improved robustness of extension updates on Windows by properly managing file handles and adding retry logic to handle file locks from OneDrive/Antivirus.

## [0.0.12] - 2026-04-27

### Added
- Testing: Triggered new version to verify the auto-update mechanism using static Release Asset URLs.

## [0.0.11] - 2026-04-27

### Added
- UI/UX: Integrated `Skeleton` loaders with shimmer animations for better perceived performance.
- UI/UX: Added micro-animations (fade, scale, slide) for smoother transitions in tabs and panels.
- Architecture: Centralized keyboard shortcuts into a dedicated `useKeyboardShortcuts` composable.
- Shortcuts: New `Ctrl+N` for connections and improved `Ctrl+K` for focusing search.

### Fixed
- UI/UX: Resolved an issue where global keyboard shortcuts blocked standard input typing.

## [0.0.8] - 2026-04-27

### Added
- UI/UX: Implemented professional 3-panel layout system with resizable and dockable panels.
- UI/UX: Introduced responsive Grid Toolbar with adaptive visibility levels (Level 1-3).
- Components: New `PanelRail` for minimized panels and `Tooltip` component for enhanced interactivity.
- Layout Store: Dedicated Pinia store for dynamic layout orchestration.

### Changed
- CI/CD: Streamlined release pipeline by removing redundant Windows icon patching.
- Security: Tightened NeutralinoJS security settings and refined global keyboard shortcuts.


## [0.0.7] - 2026-04-26

### Added
- New agent skill: `changelog-automation` for structured change tracking.
- New agent rule: `changelog.md` to mandate enriched release notes.
- Documentation: `SPEC.md`, `PROGRESS.md`, and `GEMINI.md` are now tracked in version control.

### Changed
- Assets: Renamed `mockups/` directory to `screenshots/` and updated `README.md` preview link.
- Project Progress: Synchronized `PROGRESS.md` with current milestones and roadmap.

## [0.0.6] - 2026-04-26

### Changed
- CI/CD: Updated release workflow to use `neu build --release` for automatic asset zipping.

## [0.0.5] - 2026-04-26

### Fixed
- CI/CD: Updated Go setup to use `go-version-file` to match `extensions/db-bridge/go.mod`.

## [0.0.4] - 2026-04-26

### Fixed
- CI/CD: Switched from `npm ci` to `npm install --legacy-peer-deps` to resolve dependency conflicts in the pipeline.

## [0.0.3] - 2026-04-26

### Added
- CI/CD: Initial GitHub Actions release workflow for automated builds and releases.

## [0.0.2] - 2026-04-26

### Added
- Agent Workspace: Initial setup of custom rules and workflows in `.agents/`.

## [0.0.1] - 2026-04-25

### Added
- Initial stable release with Go-based backend extension (`db-bridge`).
- Support for PostgreSQL, MySQL, and SQLite.
- Integrated SQL Editor with CodeMirror 6.
- Auto-update mechanism for frontend and backend.
