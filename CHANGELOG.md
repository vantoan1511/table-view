# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8] - 2026-04-26

### Fixed
- Assets: Fixed application icon for taskbar and Windows executable using `rcedit` in CI.


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
