# NeutralinoJS + Vue Skill

This skill provides instructions and best practices for developing applications using NeutralinoJS with a Vue 3 (TypeScript) frontend and a Go-based extension bridge.

## Architecture

The application follows a three-tier architecture:
1. **Frontend (Vue 3 + Pinia)**: Handles the UI and user interactions. Uses `@neutralinojs/lib` to interact with native APIs.
2. **Native Layer (NeutralinoJS)**: Provides access to the operating system (filesystem, window management, OS commands).
3. **Extension Bridge (Go)**: A custom binary (`extensions/db-bridge`) for heavy-duty tasks like database operations. Communicates with the frontend via Neutralino's extension system.

## NeutralinoJS Lifecycle

### Initialization
Neutralino should be initialized in the main entry point (`src/main.ts`) before the app starts interacting with native APIs.

```typescript
import * as Neutralino from '@neutralinojs/lib'

if (window.NL_PORT) {
  Neutralino.init()
  // Optional: Perform initial setup like setting window title
}
```

### Environment Checks
Always check if `window.NL_PORT` is defined before calling Neutralino APIs to ensure the app doesn't crash when running in a standard browser (e.g., during some unit tests or development scenarios).

## Core API Usage

### Filesystem (`Neutralino.filesystem`)
Used for reading/writing files and managing directories.

- **Check file existence**: `await Neutralino.filesystem.getStats(path)` (throws if not found).
- **Read file**: `await Neutralino.filesystem.readFile(path)`.
- **Write file**: `await Neutralino.filesystem.writeFile(path, content)`.
- **Remove path**: `await Neutralino.filesystem.remove(path)`.

### Window Management (`Neutralino.window`)
- **Set Title**: `await Neutralino.window.setTitle(title)`.
- **Minimize/Maximize**: `await Neutralino.window.minimize()`, `await Neutralino.window.maximize()`.
- **Show/Hide**: `await Neutralino.window.show()`, `await Neutralino.window.hide()`.

### Application (`Neutralino.app`)
- **Get Config**: `await Neutralino.app.getConfig()`.
- **Exit**: `await Neutralino.app.exit()`.
- **Restart**: `await Neutralino.app.restartProcess()`.

## Extension Communication

Extensions are dispatched using their unique ID defined in `neutralino.config.json`.

### Dispatching to Extension
```typescript
Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'eventName', payload)
```

### Listening for Extension Events
```typescript
Neutralino.events.on('extensionEventName', (evt) => {
  const data = evt.detail
  // Handle response
})
```

### Request-Response Pattern
Since extension calls are asynchronous and event-driven, use a `reqId` to match responses to requests.

```typescript
function callExtension(event, data) {
  return new Promise((resolve, reject) => {
    const reqId = Date.now().toString()
    const onResult = (evt) => {
      if (evt.detail.reqId === reqId) {
        Neutralino.events.off('responseEvent', onResult)
        if (evt.detail.success) resolve(evt.detail)
        else reject(evt.detail.error)
      }
    }
    Neutralino.events.on('responseEvent', onResult)
    Neutralino.extensions.dispatch(EXT_ID, event, { ...data, reqId })
  })
}
```

## Project Standards

### State Management (Pinia)
- Logic interacting with Neutralino or Extensions should reside in Pinia stores.
- Stores should handle initialization (e.g., `init()` method called after `Neutralino.init()`).

### Styling (Tailwind CSS 4)
- Follow the established dark-mode aesthetic.
- Use Lucide Vue Next for icons.

### Updater Logic
- Use `Neutralino.updater.checkForUpdates(manifestUrl)` to check for new versions.
- The `manifest.json` should be hosted on GitHub.
- Extension updates are handled via a custom `dbBridge.updateExtension` event to the Go backend.

## Development Workflow

### Build & Run
- **Frontend Dev**: `npm run dev` (starts Vite).
- **Neutralino Dev**: `neu run` (starts Neutralino and points to Vite dev server).
- **Build Extension**: `cd extensions/db-bridge && go build -o db-bridge.exe` (on Windows).
- **Production Build**: `neu build`.

### Configuration (`neutralino.config.json`)
- Ensure `nativeAllowList` includes all necessary API namespaces.
- Update `modes.window` for initial dimensions and properties.
- Exclude unnecessary files in `cli.resourcesPath` to keep the bundle small.
