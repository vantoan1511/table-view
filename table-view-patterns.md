---
name: table-view-patterns
description: Coding patterns extracted from table-view
version: 1.0.0
source: local-git-analysis
analyzed_commits: 100
---

# Table View Patterns

## Commit Conventions

This project uses **conventional commits**:
- `feat:` - New features (e.g., `feat(ui):`, `feat(grid):`)
- `fix:` - Bug fixes (e.g., `fix(database):`)
- `chore:` - Maintenance tasks (e.g., `chore: release v0.2.4`)
- `docs:` - Documentation updates
- `perf:` - Performance optimizations
- `refactor:` - Code refactoring
- `ci:` - CI/CD changes

## Code Architecture

### Backend (Rust)
- Location: `extensions/db-bridge/`
- All database bridge logic is written in **Rust**.
- Uses `sqlx` and dedicated drivers for PostgreSQL, MySQL, SQLite, and Oracle.

### Frontend (Vue 3 + Pinia)
- Location: `src/`
- State Management: Pinia stores located in `src/stores/`.
- Components: Organized by functional area in `src/components/`.
- Styling: Tailwind CSS 4.

## Workflows

### Database Extension
When modifying database bridge logic:
1.  Work in `extensions/db-bridge/src/drivers/`.
2.  Use `sqlx` or dedicated drivers for safety and performance.
3.  Always perform a `cargo build` to ensure the binary compiles.

### Releases
Releases involve:
1.  Updating `package.json`, `neutralino.config.json`, and `manifest.json`.
2.  Updating `CHANGELOG.md`.

## Testing Patterns
- Test data: SQL scripts in `scripts/`.
- Rust tests: Located in `extensions/db-bridge/src/test.rs` or alongside source.
