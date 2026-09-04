<div align="center">

# Table View

**Lightweight, high-performance desktop database management tool for PostgreSQL, MySQL, SQLite, and Oracle.**

[![Latest Release](https://img.shields.io/github/v/release/vantoan1511/table-view?label=version&style=flat-square&color=blue)](https://github.com/vantoan1511/table-view/releases)
[![WinGet](https://img.shields.io/badge/winget-vantoan1511.TableView-5C2D91.svg?style=flat-square&logo=windows)](https://github.com/microsoft/winget-pkgs/tree/master/manifests/v/vantoan1511/TableView)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Runtime](https://img.shields.io/badge/Runtime-Neutralinojs_v6-orange.svg?style=flat-square&logo=javascript)](https://neutralino.js.org/)
[![Frontend](https://img.shields.io/badge/Frontend-Vue_3_%2B_PrimeVue_v4-42b883.svg?style=flat-square&logo=vue.js)](https://vuejs.org/)
[![Backend](https://img.shields.io/badge/Backend-Rust_%2B_Tokio-black.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Release Status](https://img.shields.io/github/actions/workflow/status/vantoan1511/table-view/release.yml?label=release&style=flat-square)](https://github.com/vantoan1511/table-view/actions/workflows/release.yml)

<br />

<p align="center">
  <img src="preview/preview.png" alt="Table View Preview" width="880" style="border-radius: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.15);" />
</p>

</div>

---

## ⚡ Why Table View?

Traditional database clients often rely on bloated Electron runtimes that consume 500MB+ of RAM upon startup, or legacy Java applications with sluggish UI rendering. 

**Table View** takes a different path. Built with a decoupled native architecture—pairing an ultra-lean **Neutralinojs** desktop webview with a high-throughput **Rust backend extension**—Table View delivers the responsiveness of a native desktop application with minimal system resource utilization.

| Metric / Attribute | Table View | Typical Electron-Based Clients |
|:---|:---|:---|
| **Memory Footprint** | **~50 – 90 MB RAM** | 400 – 900+ MB RAM |
| **Startup Time** | **< 1 second (Instant)** | 3 – 8 seconds |
| **Backend Engine** | **Native Rust (`tokio` + `sqlx`)** | Node.js IPC / JS driver wrappers |
| **Oracle Driver** | **Pure Rust thin-driver** (Zero client libs) | Requires heavy Oracle Instant Client / OCI |
| **Credential Security**| **Local AES encryption** | Often stored in plain text or base64 |

---

## 🗄️ Supported Databases

Table View connects directly to your databases via native Rust drivers with connection pooling:

| Engine | Driver Architecture | Connection Options | Key Capabilities |
|:---|:---|:---|:---|
| **PostgreSQL** | Pure Rust (`sqlx`) | Standard TCP, SSL / TLS (`rustls`) | Full schema introspection, foreign keys, indexes, constraint inspection |
| **MySQL / MariaDB** | Pure Rust (`sqlx`) | TCP, SSL / TLS, Custom Collation | Multiple database navigation, fast paging, table altering |
| **SQLite** | Embedded Rust (`sqlx`) | Local file path (`.db`, `.sqlite`, `.sqlite3`) | Embedded file browsing, foreign key enforcement, VACUUM / analyze |
| **Oracle Database** | Pure Rust (`deadpool-oracle` / `oracle-rs`) | SID / Service Name, Port, SYSDBA / SYSOPER | **Thin driver** — requires no Oracle Instant Client or C libraries |

---

## ✨ Core Features

### 📊 Modern Data Grid
- **Virtual Scrolling**: Smoothly browse millions of rows without UI lag or memory runaway.
- **Inline Cell Editing**: Double-click any cell to edit data in-place with transactional commit and rollback support.
- **Row Insertion & Deletion**: Add new records or select and bulk-delete rows with primary-key safety checks.
- **Filtering & Multi-Column Sorting**: Instantly filter datasets by column and sort by ascending/descending order.
- **Value Viewer**: Dedicated slide-out viewer with syntax highlighting and monospaced layout for large text, JSON documents, or binary objects.

### 💻 Advanced SQL Workspace
- **CodeMirror 6 Engine**: Modern, responsive SQL query editor with syntax highlighting and autocompletion for SQL keywords, tables, and columns.
- **Query Formatter**: One-click SQL beautification powered by `sql-formatter`.
- **Telemetry & Diagnostics**: Backend-accurate query execution timers down to the millisecond (`ms`) displayed in the Status Bar.
- **Multi-Tab Workspace**: Open and manage multiple query tabs and data grids concurrently with persistent state.
- **Execution History & Console**: Track query execution history, execution durations, affected row counts, and detailed error logs.

### 🔍 Schema Explorer & Table Management
- **Tree Navigation**: Comprehensive sidebar hierarchy covering connections, databases, schemas, tables, and views.
- **Properties & Constraint Inspector**: Inspect column data types, nullability, default values, primary keys, foreign keys, and indexes.
- **Table Alterations (DDL)**: Visually add columns, drop columns, and rename columns without writing manual `ALTER TABLE` statements.
- **CSV Data Export**: Export entire tables or filtered datasets directly to CSV.

### 🛡️ Security & Developer Experience
- **Secure Credential Vault**: Database passwords are encrypted locally using AES-GCM before persistence—never stored in cleartext.
- **Dark & Light Modes**: Restrained, IDE-inspired UI based on PrimeVue v4 with Aura preset and Tailwind CSS v4.
- **Connection Color Tags**: Assign semantic color tags (Production, Staging, Dev) to avoid accidental modifications on critical environments.
- **Import / Export Profiles**: Backup and share connection profiles securely across machines.
- **Integrated Auto-Updater**: Automatic background update notifications powered by GitHub Releases.

---

## 🏗️ Architecture

Table View separates UI concerns from privileged system operations via a strict IPC contract:

```
┌─────────────────────────────────────────────────────────────┐
│                       Vue 3 Frontend                        │
│   - PrimeVue v4 + Tailwind CSS v4 UI Components             │
│   - CodeMirror 6 SQL Editor & Virtual DataGrid              │
│   - Pinia Reactive State (Connections, Tabs, Preferences)   │
└──────────────────────────────┬──────────────────────────────┘
                               │ Neutralinojs WebSocket IPC
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    Neutralinojs Runtime                     │
│   - Lightweight Native OS Webview                           │
│   - OS-level Window, File System, and Storage APIs          │
└──────────────────────────────┬──────────────────────────────┘
                               │ JSON-RPC Extension Bridge
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                Rust Native Extension (`db-bridge`)          │
│   - Tokio Asynchronous Multi-threaded Runtime               │
│   - Native Connection Pooling (`sqlx`, `deadpool-oracle`)   │
│   - Direct TCP Sockets & SSL/TLS Database Drivers           │
└──────────────────────────────┬──────────────────────────────┘
                               │ Direct TCP / Native Drivers
                               ▼
        PostgreSQL  •  MySQL  •  SQLite  •  Oracle
```

---

## 🚀 Getting Started

### Installation for End Users

#### Option 1: Via Windows Package Manager (WinGet)
Install the latest official release directly from PowerShell or Command Prompt:

```bash
winget install vantoan1511.TableView
```

#### Option 2: Direct Windows Installer
1. Download the latest `setup.exe` from the [GitHub Releases](https://github.com/vantoan1511/table-view/releases) page.
2. Run the installer and launch **Table View**.

> [!NOTE]
> **Microsoft Defender SmartScreen:**  
> Because Table View is an independent open-source project and is not signed with an expensive commercial EV certificate, Windows SmartScreen may display a *"Windows protected your PC"* prompt on initial launch.  
> Click **"More info"** and select **"Run anyway"** to continue. You can independently verify release binary integrity via the published `SHA256SUMS.txt` file on GitHub Releases.

---

## 🛠️ Development Setup (Building from Source)

### Prerequisites
- **Node.js**: `v20.19.0` or `>=22.12.0`
- **Rust toolchain**: Stable 1.80+ (`cargo`, `rustc`)
- **Neutralinojs CLI**: Install globally via npm:
  ```bash
  npm install -g @neutralinojs/neu
  ```

### 1. Clone & Install Dependencies
```bash
# Clone the repository
git clone https://github.com/vantoan1511/table-view.git
cd table-view

# Install frontend dependencies
npm install
```

### 2. Build the Rust Backend Extension
The Rust backend lives in `extensions/db-bridge`. You can build and bundle it using the automated script:

```bash
node scripts/build-ext.js
```
*This compiles `extensions/db-bridge` in release mode and copies `db-bridge.exe` into the `bin/` directory.*

### 3. Run in Development Mode
Launch the application with live frontend hot-reload and active Neutralino runtime:

```bash
npm run neu:run
```
*(Or run `neu run` directly).*

### 4. Running Tests & Quality Checks
```bash
# Run unit tests (Vitest)
npm run test:run

# Run TypeScript type check
npm run type-check

# Run linter
npm run lint

# Check Rust backend
cd extensions/db-bridge
cargo check
cargo test
```

### 5. Packaging Production Binaries & Installer
```bash
# Build production bundle and clean artifacts
npm run neu:build

# Generate Windows Inno Setup installer
npm run package
```
*The installer will be generated under `Output/setup.exe`.*

---

## ⌨️ Keyboard Shortcuts

Table View provides standard desktop keyboard shortcuts for maximum speed:

| Action | Shortcut (Windows / Linux) | Shortcut (macOS) |
|:---|:---|:---|
| **Run Query** | <kbd>Ctrl</kbd> + <kbd>Enter</kbd> | <kbd>Cmd</kbd> + <kbd>Enter</kbd> |
| **New Connection** | <kbd>Ctrl</kbd> + <kbd>N</kbd> | <kbd>Cmd</kbd> + <kbd>N</kbd> |
| **Focus Search** | <kbd>Ctrl</kbd> + <kbd>K</kbd> | <kbd>Cmd</kbd> + <kbd>K</kbd> |
| **Close Active Tab** | <kbd>Ctrl</kbd> + <kbd>W</kbd> | <kbd>Cmd</kbd> + <kbd>W</kbd> |
| **Refresh Data / Schema** | <kbd>Ctrl</kbd> + <kbd>R</kbd> or <kbd>F5</kbd> | <kbd>Cmd</kbd> + <kbd>R</kbd> or <kbd>F5</kbd> |
| **Toggle Sidebar** | <kbd>Ctrl</kbd> + <kbd>B</kbd> | <kbd>Cmd</kbd> + <kbd>B</kbd> |
| **Toggle Console / Logs** | <kbd>Ctrl</kbd> + <kbd>J</kbd> | <kbd>Cmd</kbd> + <kbd>J</kbd> |
| **Toggle Properties Inspector** | <kbd>Ctrl</kbd> + <kbd>I</kbd> | <kbd>Cmd</kbd> + <kbd>I</kbd> |
| **Preferences** | <kbd>Ctrl</kbd> + <kbd>,</kbd> | <kbd>Cmd</kbd> + <kbd>,</kbd> |
| **Shortcuts Reference** | <kbd>Ctrl</kbd> + <kbd>/</kbd> | <kbd>Cmd</kbd> + <kbd>/</kbd> |

---

## 📂 Project Structure

```
table-view/
├── bin/                       # Bundled native extension binaries (db-bridge.exe)
├── extensions/
│   └── db-bridge/             # Rust native database connector
│       ├── src/
│       │   ├── drivers/       # Postgres, MySQL, SQLite, Oracle implementations
│       │   ├── handler.rs     # Command router and IPC message handling
│       │   ├── pool.rs        # Connection pool management
│       │   └── main.rs        # WebSocket client entry point
│       └── Cargo.toml
├── public/                    # Static assets & application icons
├── scripts/                   # Build scripts (installer, ext builder, release notes)
├── src/                       # Vue 3 application source code
│   ├── assets/                # Design tokens (base.css) and global styling
│   ├── components/            # UI components (grid, sql, layout, modals, panels)
│   ├── composables/           # Reusable Vue composables (shortcuts, tab sync)
│   ├── services/              # NativeService (OS/IPC) & BridgeService (Rust DB)
│   ├── stores/                # Pinia stores (connections, grid, tabs, preferences)
│   ├── theme/                 # PrimeVue Aura theme preset customizations
│   └── types/                 # TypeScript interfaces and connection models
├── neutralino.config.json     # Neutralino runtime configuration
└── package.json
```

---

## 🤝 Contributing

Contributions are welcome! Whether it's reporting a bug, proposing new database drivers, or submitting improvements:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/my-feature`).
3. Commit your changes following [Conventional Commits](https://www.conventionalcommits.org/) (`git commit -m 'feat: add support for CockroachDB'`).
4. Push to your branch (`git push origin feature/my-feature`).
5. Open a Pull Request.

Please check open issues on [GitHub Issues](https://github.com/vantoan1511/table-view/issues) before initiating large refactors or new features.

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.
