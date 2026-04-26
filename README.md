# Table View

**Table View** is a lightweight, high-performance desktop database management tool built with [NeutralinoJS](https://neutralino.js.org/) and [Vue 3](https://vuejs.org/). It provides a unified interface to browse, edit, and manage multiple database types (PostgreSQL, MySQL, SQLite) with a focus on speed and a premium user experience.

![Table View Preview](https://raw.githubusercontent.com/vantoan1511/table-view/main/screenshots/preview.png)

## ✨ Features

- **Multi-DB Support**: Connect to PostgreSQL, MySQL, and SQLite databases simultaneously.
- **Rich Data Grid**: 
  - **Inline Editing**: Double-click any cell to edit data instantly.
  - **Bulk Operations**: Select multiple rows for batch deletion.
  - **Auto-Generating UUIDs**: Intelligent handling of PKs and UUID columns during row insertion.
  - **Understandable Types**: Automatically translates internal DB type OIDs into human-readable text.
- **Schema Management**:
  - **Visual Schema Tree**: Browse tables, views, functions, and schemas.
  - **Alter Table UI**: Add, rename, or drop columns via a dedicated structure management dialog.
- **SQL Editor**: Write and execute raw SQL queries with syntax highlighting and autocompletion (powered by CodeMirror 6).
- **Modern UX**:
  - **Global Error Handling**: Uncaught exceptions are displayed in a polished, detailed dialog rather than silently failing.
  - **Toast Notifications**: Beautiful, non-intrusive feedback for system actions.
  - **Dark Mode Aesthetic**: A sleek, dark-themed interface designed for professional developers.
  - **Custom Context Menus**: Native-feeling menus for quick actions on tables and connections.

## 🛠️ Tech Stack

- **Core Runtime**: [NeutralinoJS](https://neutralino.js.org/) (Cross-platform desktop framework)
- **Frontend**: [Vue 3](https://vuejs.org/) with [Pinia](https://pinia.vuejs.org/) (State management)
- **Styling**: [Tailwind CSS 4](https://tailwindcss.com/)
- **Icons**: [Lucide Vue Next](https://lucide.dev/)
- **Editor**: [CodeMirror 6](https://codemirror.net/)
- **Backend Bridge**: High-performance Go-based Neutralino extension for database drivers (pgx, mysql, modernc-sqlite).

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v20 or later)
- [Neutralinojs CLI](https://neutralino.js.org/docs/cli/neu-cli) (`npm install -g @neutralinojs/neu`)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/vantoan1511/table-view.git
   cd table-view
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

### Running in Development

Start the development server with hot-reload and Neutralino runtime:
```bash
neu run
```

### Building for Production

To package the application for Windows, Linux, and macOS:
```bash
neu build
```
The binaries will be available in the `dist/` directory.

## 📂 Project Structure

- `src/`: Vue 3 application source code.
  - `components/`: UI components (Grid, Sidebar, SQL Editor, etc.).
  - `stores/`: Pinia state management modules.
  - `assets/`: Global styles and static assets.
- `extensions/`: Neutralino extensions.
  - `db-bridge/`: The core database connector written in Go.
- `neutralino.config.json`: Neutralino application configuration.

## 📄 License

This project is open-source and available under the [MIT License](LICENSE).
