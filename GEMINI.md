# GEMINI.md

# Project

Table View is a lightweight, native desktop database management tool.

Technology stack:

- Rust (backend)
- Neutralinojs (desktop runtime)
- Vue 3
- Tailwindcss v4
- PrimeVue v4
- TypeScript
- Vite

The frontend communicates with the Rust backend exclusively through Neutralinojs IPC and the db-bridge Extension WebSocket. Rust is responsible for all privileged operations, including direct database connections and query execution, while Vue is responsible for presenting data and handling user interactions.

---

# Agent Workflow

Agents must follow this workflow for every task.

## Phase 1 — Understand

Before modifying code:

1. Use `codegraph` (`codegraph_explore`) as the primary tool to explore architecture, survey components, and locate relevant symbols/flows.
2. Inspect the relevant existing implementation and neighboring components for established patterns.
3. Search for reusable components, composables, services, utilities, types, and IPC/bridge commands.
4. Inspect relevant design-system files for UI work.
5. Identify the smallest set of files that should change.

Do not begin implementation before completing this phase.

## Phase 2 — Plan

For non-trivial tasks:

1. Following workflow `/plan` to create or update the corresponding plan.
2. Record:
   - problem
   - current implementation
   - proposed solution
   - files to modify
   - files to create
   - reusable existing code
   - risks
   - verification steps
3. Ensure the plan is based on repository evidence discovered during Phase 1.

Do not invent files, APIs, components, or architecture.

## Phase 3 — Implement

Implement only the requested scope.

- Reuse existing code before creating new abstractions.
- Follow established patterns before introducing new patterns.
- Keep changes focused and minimal.
- Do not refactor unrelated code.
- Do not add speculative features.
- Do not change architecture unless required by the task.
- Do not create local variants of shared components without a clear semantic reason.

## Phase 4 — Verify

After implementation:

1. Run relevant tests.
2. Run type checking (`npm run type-check` or `vue-tsc`).
3. Run linting when configured (`npm run lint`).
4. Build the affected application/package when practical (`cargo build`, `npm run build`).
5. Inspect `git diff` and `git status`.
6. Verify that no unrelated files were modified.

If a verification step cannot be performed, explicitly report why.

Never claim verification that was not actually performed.

## Phase 5 — Report

The final response must contain:

- What changed
- Files changed
- Verification performed
- Remaining issues or limitations

Never claim that work was completed if required verification has not been performed.

---

# Repository Evidence

The repository is the source of truth.

Before referring to an existing component, composable, service, utility, IPC command, Rust module, type, route, store, or design token, locate and verify it in the repository using `codegraph` (`codegraph_explore`) or targeted searches.

Never assume an implementation exists because it would be conventional.

If an API or abstraction cannot be found:

1. Explore using `codegraph_explore` or search again using related terminology.
2. Confirm that it does not exist.
3. Propose the smallest required addition rather than assuming it exists.

Use existing implementation patterns as evidence for new code whenever possible.

---

# Scope Discipline

Implement the requested change, not an imagined future version of it.

Do not:

- add speculative abstractions
- redesign unrelated UI
- rename unrelated variables
- reorganize directories without need
- upgrade dependencies unless requested or strictly required
- introduce additional features
- "clean up" unrelated code
- refactor neighboring code merely because it could be improved

If an improvement is discovered outside the requested scope, mention it in the final report instead of implementing it.

---

# Reuse Hierarchy

When implementing functionality, prefer solutions in this order:

1. Existing feature implementation
2. Existing shared component
3. Existing composable, service, or utility
4. Existing PrimeVue component
5. New feature-local implementation
6. New shared abstraction only when multiple consumers justify it

Do not create a shared abstraction merely because code could theoretically be reused.

---

# Git Workflow

For feature work and bug fixes:

1. Do not work directly on `main` or `master`.
2. Before modifying files, verify the current branch.
3. Create or checkout a dedicated branch before making changes.
4. Keep the branch scoped to the task.
5. Do not commit unrelated changes.
6. Do not rewrite existing commits unless explicitly requested.
7. Before finishing, inspect `git diff` and `git status`.

---

# Coding Rules

Before generating code:

1. Inspect the existing architecture.
2. Follow established project conventions.
3. Reuse existing modules whenever possible.
4. Keep changes focused and minimal.
5. Avoid unrelated refactoring.
6. Never perform self-initiated or unrequested refactoring; touch existing code only when explicitly asked or strictly required for the requested change.
7. Explain trade-offs when multiple implementations are reasonable.
8. Always check-out to another branch for features implementing or bugs fixing, never work on main/master.

Never invent APIs that do not exist.

If a required API is missing, propose adding it rather than assuming it already exists.

---

# Mission

Table View should be:

- Lightweight
- Fast
- Native-feeling
- Reliable
- Secure
- Cross-platform
- Easy to maintain
- Production ready

Every implementation should favor simplicity, predictability, and long-term maintainability.

---

# Architecture

Table View is divided into two layers.

## Backend (Rust)

The Rust backend owns:

- Database connections and pooling
- Query execution
- File system access
- Network access
- Background tasks
- Business logic
- Performance-critical operations
- System integration

The backend should not contain UI concerns.

## Frontend (Vue)

The frontend owns:

- Rendering
- User interactions
- View state
- Routing
- Local UI state
- Animations

The frontend should avoid implementing business logic.

Whenever logic requires database knowledge or system access, it belongs in Rust.

---

# Communications (IPC & WebSockets)

The IPC/WebSocket boundary is the contract between frontend and backend.

Always:

- keep commands small
- use strongly typed request/response models
- return structured errors
- avoid sending unnecessary data
- keep payloads versionable

Do not expose internal backend implementation details through the bridge.

Treat the extension communication layer as a stable public API.

---

# Single Source of Truth

Business rules must exist in one place only.

Avoid duplicating logic between Rust and TypeScript.

If validation or calculations are required, prefer implementing them in Rust and exposing the results through the backend bridge.

---

# Rust Guidelines

Prefer:

- ownership over unnecessary cloning
- explicit error handling
- Result<T, E>
- idiomatic Rust
- modular crates
- strong typing

Avoid:

- unwrap()
- expect() outside tests
- panic! for recoverable errors
- unnecessary Arc<Mutex<T>>
- global mutable state

Prefer immutable data structures whenever practical.

---

# Vue Guidelines

Use:

- Composition API
- script setup
- TypeScript
- composables
- reusable components
- PrimeVue v4 components (`Button`, `InputText`, `InputNumber`, `Select`, `ToggleSwitch`, etc.) over raw HTML elements or custom controls
- centralized service wrappers (e.g., `src/services/bridge.ts`, `src/services/nativeService.ts`) for all native or system operations

Avoid:

- Options API
- large components
- business logic inside views
- direct bridge/IPC calls scattered across components
- direct imports of `@neutralinojs/lib` in components, views, or composables (must use frontend services)
- raw HTML inputs/buttons (`<button>`, `<input>`, `<select>`) or reinventions of controls already available in PrimeVue

Components should remain focused on rendering.

---

# Workflow Mandates

- **Validation**: After modifying the Rust extension, always perform a `cargo build` to ensure the binary compiles correctly before testing with NeutralinoJS.
- **Logging**: Maintain a centralized log file in a writable system directory (e.g., `%LOCALAPPDATA%` on Windows), capturing both backend (Rust) and frontend (UI) diagnostic information for unified troubleshooting.
- **Pathing**: Use Windows-compatible backslash paths (`\\`) in `neutralino.config.json` for extension commands.

---

# UI/UX Standards

- **Icons**: Use Lucide Vue Next. Always import icons explicitly in the `<script setup>` block.
- **Feedback**: Use the existing `toast` and `error` stores to provide immediate user feedback for all background operations.
- **Confirmation Dialog**: Always use the existing `ConfirmDialog` component. Never use browser native `confirm()` or `alert()`.

---

# Services

Frontend services should only wrap bridge/IPC calls.

Example flow:

View
↓
Composable
↓
Frontend Service (e.g., `src/services/bridge.ts` or `src/services/native.ts`)
↓
Neutralino IPC / Extension WebSocket
↓
Rust Backend
↓
Database

Components must never call IPC or WebSockets directly. All native/OS/filesystem/IPC capabilities must be accessed via frontend services.

---

# State Management

Keep state local whenever possible.

Shared state should only exist when genuinely shared across multiple views.

Do not duplicate backend state inside multiple frontend stores.

---

# Database Operations

The Rust backend owns every interaction with the databases.

Never access a database directly from the frontend.

Support:

- multiple connection profiles
- multiple database engines (PostgreSQL, MySQL, SQLite, Oracle)
- schema and table introspection
- concurrent query execution
- proper connection pooling

Never assume:

- default schema
- specific SQL dialects in common abstraction layers
- specific database version

---

# Security

Sensitive information never belongs in the frontend.

Do not expose:

- database credentials
- connection strings with passwords
- private keys

The frontend should only receive the minimum information necessary to render the UI.

---

# Error Handling

Errors returned through the backend bridge should:

- include machine-readable error codes
- contain user-friendly messages
- avoid leaking implementation details

Unexpected failures should be logged by the backend.

---

# Performance

Prefer:

- streaming or incremental updates
- background workers
- caching
- lazy loading
- batching requests

Avoid unnecessary serialization across the backend bridge.

---

# Dependencies

Before adding dependencies:

Evaluate:

- maintenance
- security
- compile time
- binary size
- community adoption

Prefer standard library functionality whenever practical.

Avoid introducing dependencies for small utilities.

---

# Code Style

Write code that is easy to understand.

Prefer descriptive names.

Avoid abbreviations.

Good:

- databaseConnection
- activeSchema
- tableMetadata

Avoid:

- ctx
- obj
- tmp
- data1

Variable Naming in CSS

Good:

- border-(--color-border)
- min-w-35
- shrink-0

Avoid:

- border-[var(--color-border)]
- min-w-[140px]
- flex-shrink-0

**Additional Coding Conventions:**

- **Function Syntax**: Use arrow functions for composables, callbacks, and inline handlers. Do not refactor existing standard function declarations unless modifying them.
- **Import Order**: Group imports into standard categories with blank lines between groups:
  1. Vue core & external libraries
  2. Pinia stores & composables
  3. UI & feature components
  4. Types & utilities
- **Component Reuse**: Check `src/components/ui` and existing modal/panel components before creating new UI primitives.

---

# Comments

Comments should explain why.

Avoid comments that simply describe what the code is doing.

---

# Logging

Backend:

- log diagnostics
- log failures
- avoid sensitive data
- Always maintain centralized logging via Rust `tracing` loggers rather than bare `println!`.

Frontend:

- avoid excessive console logging
- never log secrets
- Always maintain centralized logging via `logger.ts` rather than bare `console.log`.

---

# Documentation

Whenever a public backend bridge command changes:

Update:

- bridge documentation
- request schema
- response schema
- types

Documentation should reflect implementation.

---

# UI Design System

Table View's visual language is **technical, compact, information-dense, and restrained**. It must feel like a high-performance native desktop database client or IDE — not a consumer SaaS dashboard.

The canonical source of truth for all visual decisions is:

- `src/assets/base.css` — design tokens (colors, shadows, radii, typography)
- `src/assets/main.css` — global base styles, scrollbar, layout resets
- `src/theme/TableViewTheme.ts` — PrimeVue preset customizations

Before adding any CSS, always check these files first.

---

## UI Principles

### Information Density

Prefer useful information over decorative whitespace while preserving scanability and clear hierarchy.

### Hierarchy

Establish hierarchy in this order:

1. Typography
2. Spacing
3. Surface contrast
4. Semantic color
5. Borders only when structural

Do not use every mechanism simultaneously for the same element.

### Progressive Disclosure

Show information needed for the current task first. Move secondary information into side panels, drawers, popovers, tabs, or contextual overlays.

### Interaction Locality

Actions should appear near the resource or state they affect (e.g. table toolbar actions next to the active data table).

### Consistency

Before creating or modifying a UI component:

1. Search for existing instances of the same component or interaction.
2. Reuse the established implementation.
3. If the component is shared, modify the shared component instead of creating a local variant.
4. Do not introduce a new visual variant without a clear semantic reason.

---

## Typography

### Fonts

- UI text: `--font-ui` (`Inter`, `Segoe UI`, system-ui, sans-serif)
- Monospace: `--font-mono` (`JetBrains Mono`, `Cascadia Code`, `Fira Code`, monospace)

Use `font-ui` for all prose, labels, navigation, and standard controls.
Use `font-mono` for SQL query editors, raw data cells, execution plans, and terminal-style output.

### Text Scale

| Usage | Size class | Weight |
|---|---|---|
| Page/section title | `text-xl` | `font-bold` |
| Modal/drawer/tab title | `text-lg` | `font-bold` |
| Table headers, form labels | `text-sm` | `font-medium` or `font-semibold` |
| Grid data cell content | `text-xs` | `font-normal` |
| Footer / status bar | `text-[11px]` | `font-medium` |
| Badges / tags | `text-xs` | `font-semibold` |

---

## Color & Surface Tokens

Use CSS custom properties from `base.css`. Never hard-code hex values that duplicate an existing token.

- Semantic colors (`--color-success`, `--color-danger`, `--color-warning`) communicate state only. Never use them decoratively.
- Accent/Primary color (`--color-primary`) is reserved for interactive focus, primary actions, and selected items.
- Surfaces (`--color-surface`, `--color-sidebar`, `--color-border`) create clean structure without heavy styling.

---

## Component Grouping Without Borders

When spacing alone is insufficient to distinguish complex component groups, use these **borderless grouping techniques**:

1. **Proportional Ratio Spacing (1:3 Rhythm)**:
   - Intra-item gap (between label and input): **tight** (`gap-1.5` / `gap-2`).
   - Inter-item gap (between controls in the same sub-group): **medium** (`gap-3` / `gap-4`).
   - Inter-group gap (between major sections): **wide** (`gap-8` / `gap-10`).
2. **Subtle Surface Tone Shifts (Zonal Backgrounds)**:
   - Use flat, borderless background fills (`bg-(--color-hover)`, `bg-surface-50`, `bg-surface-900`) with soft radii to group related controls without stroke lines.
3. **Typographic Rhythm & Eyebrow Headers**:
   - Use small, uppercase tracked overlines (`text-xs font-semibold tracking-wider text-muted-color`) to demarcate section boundaries clearly.
4. **Grouped Inset Wells**:
   - Group repeatable items (e.g. Connection parameters, Column definition lists, Index lists) in borderless inset wells with unified inner padding.

---

## Borders and Elevation

- Borders are structural, not decorative.
- Data tables use a flat, single-container frame (`border border-(--color-border) rounded-lg overflow-hidden bg-(--color-surface)`). Header rows use subtle zonal tone shifts with a crisp bottom separator. Numeric/metric columns should be strictly right-aligned.
- Do NOT use borders, card wrappers, or `<hr>` lines to group in-page content or divide form sections. Use natural vertical spacing (`gap-6`, `gap-8`) instead.
- Shadows are used sparingly (only `--shadow-sm` for subtle lifts or `--shadow-modal` for dialogs).

---

## PrimeVue Component Foundation

Use PrimeVue v4 components for interactive controls when an equivalent component exists:

- Button → `Button`
- Text input → `InputText`
- Number input → `InputNumber`
- Select → `Select`
- Toggle → `ToggleSwitch`
- Dialog → `Dialog`
- Confirmation → `ConfirmDialog`

Do not recreate a PrimeVue control with custom HTML/CSS without a concrete reason.

---

## Definition of Done

A task is complete only when:

- The requested behavior is implemented.
- Existing architecture and conventions are followed.
- Existing reusable components and abstractions were considered.
- No unnecessary files or abstractions were introduced.
- Relevant tests pass.
- Type checking passes (`npm run type-check` or `vue-tsc`).
- Linting passes when configured (`npm run lint`).
- The affected application/package builds successfully when applicable (`cargo build`, `npm run build`).
- The final diff contains only task-related changes.
- UI changes follow the Table View design system.
- No existing API was assumed without repository evidence.
- Verification results and remaining limitations are reported honestly.

---

# Goal

Every contribution should move Table View closer to being a professional-grade native database management desktop application with a clear separation between the Rust backend and the Vue frontend, connected through a stable, well-defined bridge interface.
