# Gemini CLI Mandates - Table View Project

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

# Coding Rules

Before generating code:

1. Inspect the existing architecture.
2. Follow established project conventions.
3. Reuse existing modules whenever possible.
4. Keep changes focused and minimal.
5. Avoid unrelated refactoring.
6. Explain trade-offs when multiple implementations are reasonable.
7. Always check-out to another branch for features implementing or bugs fixing, never work on main/master.

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

Avoid:

- Options API
- large components
- business logic inside views
- direct bridge/IPC calls scattered across components

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

---

# Services

Frontend services should only wrap bridge/IPC calls.

Example flow:

View
↓
Composable
↓
Frontend Service (e.g., `bridge.ts` or `native.ts`)
↓
Neutralino IPC / Extension WebSocket
↓
Rust Backend
↓
Database

Components should never call IPC or WebSockets directly.

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

- border-(--border)
- min-w-35
- shrink-0

Avoid

- border-[var(--border)]
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
- **Confirmation Dialog**: Always use the existing `ConfirmDialog` component. Never use browser native `confirm()` or `alert()`.

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

# Goal

Every contribution should move Table View closer to being a professional-grade native database management desktop application with a clear separation between the Rust backend and the Vue frontend, connected through a stable, well-defined bridge interface.
