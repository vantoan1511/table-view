# NeutralinoJS Skill (Full Reference)

NeutralinoJS is a lightweight, cross-platform application development framework that allows you to build native applications with web technologies (HTML, CSS, JS).

## Core Concepts

- **Client-Server Architecture**: NeutralinoJS runs a small local server that communicates with the frontend via WebSockets.
- **Client Library**: The `neutralino.js` file (or `@neutralinojs/lib` for NPM) provides the JS interface to native APIs.
- **Native APIs**: Grouped into namespaces like `os`, `filesystem`, `window`, etc.

---

## Global Variables

Predefined variables available in the `window` scope:

| Variable | Description |
| :--- | :--- |
| `NL_OS` | Operating system: `Linux`, `Windows`, or `Darwin` |
| `NL_ARCH` | CPU architecture: `x64`, `arm`, `itanium`, `ia32`, or `unknown` |
| `NL_APPID` | Application identifier (from config) |
| `NL_APPVERSION` | Application version (from config) |
| `NL_PORT` | Application port |
| `NL_MODE` | Application mode: `window`, `browser`, `cloud`, or `chrome` |
| `NL_VERSION` | NeutralinoJS framework version |
| `NL_CVERSION` | NeutralinoJS client version |
| `NL_CWD` | Current working directory |
| `NL_PATH` | Application path |
| `NL_DATAPATH` | Application data path |
| `NL_ARGS` | Command-line arguments passed to the app |
| `NL_PID` | Current process ID |
| `NL_RESMODE` | Resource source: `bundle` or `directory` |
| `NL_EXTENABLED` | `true` if extensions are enabled |
| `NL_LOCALE` | System locale (e.g., `en_US.UTF8`) |

---

## Configuration (`neutralino.config.json`)

The central configuration file for NeutralinoJS.

### Primary Fields
- `applicationId`: Unique ID (e.g., `com.example.app`).
- `version`: App version.
- `defaultMode`: Default startup mode (`window`, `browser`, `cloud`, `chrome`).
- `url`: Entry URL (e.g., `/` for `index.html`).
- `documentRoot`: Path to resources (usually `/resources/`).

### Security
- `nativeAllowList`: Array of allowed API methods (e.g., `["os.*", "filesystem.readFile"]`).
- `nativeBlockList`: Array of blocked API methods.
- `tokenSecurity`: `one-time` (recommended) or `none`.

### Window Configuration (`modes.window`)
- `title`: Window title.
- `width`, `height`: Initial dimensions.
- `minWidth`, `minHeight`, `maxWidth`, `maxHeight`.
- `center`: Boolean, centers window on startup.
- `fullScreen`, `maximize`, `hidden`, `borderless`, `resizable`.
- `alwaysOnTop`: Boolean.
- `enableInspector`: Enables DevTools.
- `icon`: Path to PNG icon.
- `exitProcessOnClose`: If `true`, app exits when window is closed.

### CLI Configuration (`cli`)
- `binaryName`: Name of the output executable.
- `resourcesPath`: Path to resources.
- `clientLibrary`: Filename of the JS library.
- `distributionPath`: Path for build output (default `/dist`).

---

## CLI Commands (`neu`)

- `neu create <name>`: Create a new project.
- `neu run`: Run the application in development mode.
- `neu build`: Build the application for all supported platforms.
- `neu update`: Update NeutralinoJS binaries and client library.
- `neu version`: Show Neutralino CLI version.

---

## API Reference

### `Neutralino.os` (Operating System)
- `execCommand(command, options)`: Execute a command and get output.
- `spawnProcess(command)`: Run a long-running process (returns `id`).
- `updateSpawnedProcess(id, action, data)`: Send `stdIn` or `exit` to process.
- `getEnv(key)` / `getEnvs()`: Get environment variables.
- `showOpenDialog(title, options)`: Open file dialog.
- `showSaveDialog(title, options)`: Save file dialog.
- `showFolderDialog(title)`: Select folder dialog.
- `showNotification(title, content, icon)`: System notifications.
- `showMessageBox(title, content, choice, icon)`: Modal dialogs.
- `setTray(options)`: Create/update system tray icon and menu.
- `open(url)`: Open a URL in the default browser.

### `Neutralino.window` (Native Window)
- `setTitle(title)` / `getTitle()`: Manage window title.
- `minimize()` / `unminimize()` / `maximize()` / `unmaximize()`.
- `show()` / `hide()` / `focus()`.
- `setSize(options)` / `getSize()`.
- `move(x, y)` / `getPosition()`.
- `setAlwaysOnTop(bool)`.
- `setBorderless(bool)`.
- `setDraggableRegion(domId)`: Make a DOM element draggable for moving the window.
- `create(url, options)`: Create a new native window (multi-window support).

### `Neutralino.filesystem` (Files & Directories)
- `createDirectory(path)`.
- `remove(path)`: Recursive removal of files/folders.
- `writeFile(path, data)` / `readFile(path)`.
- `writeBinaryFile(path, buffer)` / `readBinaryFile(path)`.
- `readDirectory(path)`: Returns entries with `entry` and `type` (`FILE`/`DIRECTORY`).
- `copy(src, dest)` / `move(src, dest)`.
- `getStats(path)`: Returns `size`, `isFile`, `isDirectory`, `modifiedAt`.
- `createWatcher(path)`: Listen for file changes (emits `watchFile` event).

### `Neutralino.storage` (Key-Value Store)
- `setData(key, value)` / `getData(key)`.
- `removeData(key)` / `getKeys()`.
- `clear()`: Wipes the entire storage.
*Note: Data is persisted in the `.storage` directory.*

### `Neutralino.events` (Native Events)
- `on(eventName, handler)`: Register for events.
- `off(eventName, handler)`: Unregister.
- `dispatch(eventName, data)`: Trigger event in current window.
- `broadcast(eventName, data)`: Trigger event in all windows and extensions.

**Core Events:**
- `ready`: App is connected to the server.
- `windowClose`: Window is closing.
- `trayMenuItemClicked`: Tray menu interaction.
- `extensionReady`: An extension is connected.

### `Neutralino.extensions` (Backend Extensions)
- `dispatch(extensionId, event, data)`: Send message to an extension.
- `broadcast(event, data)`: Send message to all extensions.
- `getStats()`: Get loaded and connected extensions.

---

## Extensions

Extensions allow you to write backend logic in any language (Go, Python, Node.js, etc.).
- **Activation**: Set `"enableExtensions": true` and define extensions in `neutralino.config.json`.
- **Connection**: Extensions connect via WebSocket using the port and token provided via CLI arguments.

---

## Application Modes

1. **window**: Native window using the system webview (default).
2. **browser**: Runs in the user's default web browser.
3. **chrome**: Runs in a dedicated Google Chrome instance (app mode).
4. **cloud**: For remote deployment (server-side Neutralino).

---

## Distribution

Run `neu build` to generate:
- `dist/<app-name>/`: Contains binaries for Windows, Linux, and macOS.
- `resources.neu`: Bundled app resources (encrypted/compressed).
