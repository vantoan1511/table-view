# Gemini CLI Mandates - Table View Project

## Core Engineering Standards
- **Backend Architecture**: All backend bridge logic must reside in the `extensions/db-bridge` directory and be written in **Go**.
- **Database Driver Guidelines**: 
  - Prefer pure Go drivers (like `modernc.org/sqlite`) to maintain easy cross-platform compilation.
  - For PostgreSQL, always use `simple_protocol` to avoid cached plan invalidation errors during schema changes.
- **Frontend State**: Use Pinia for global state. Ensure WebSocket response handling in stores matches the expected payload structures (e.g., `payload.schema` vs direct flattening).
- **Styling**: Maintain the established dark-mode aesthetic using Tailwind CSS 4.
- **Bundling**: Always update `neutralino.config.json` exclusion patterns when adding new source files or directories to keep the production bundle lean.
- **Auto-Update**:
  - For extension updates, use the `dbBridge.updateExtension` event to trigger the custom "Fresh Updater" logic in the Go backend.
  - For frontend updates, use the standard `Neutralino.updater.install()` method.
  - Maintain the update manifest URL in the `updater` store pointing to the `main` branch on GitHub (`https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest.json`).
  - Ensure version strings follow Semantic Versioning (SemVer).

## Workflow Mandates
- **Validation**: After modifying the Go extension, always perform a `go build` to ensure the binary compiles correctly before testing with NeutralinoJS.
- **Logging**: Maintain file-based logging (`db-bridge.log`) next to the extension binary for diagnostic purposes.
- **Pathing**: Use Windows-compatible backslash paths (`\\`) in `neutralino.config.json` for extension commands.

## UI/UX Standards
- **Icons**: Use Lucide Vue Next. Always import icons explicitly in the `<script setup>` block.
- **Feedback**: Use the existing `toast` and `error` stores to provide immediate user feedback for all background operations.
