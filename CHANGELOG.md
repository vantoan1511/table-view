# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
