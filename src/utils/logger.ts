import * as Neutralino from '@neutralinojs/lib';

type LogLevel = 'info' | 'warn' | 'error' | 'debug';

interface QueuedLog {
  level: LogLevel;
  message: string;
}

let isExtensionConnected = false;
const logQueue: QueuedLog[] = [];

/**
 * Sends a log message to the backend extension.
 * If the extension is not yet connected, logs are queued.
 */
async function sendToExtension(level: LogLevel, message: string) {
  if (isExtensionConnected) {
    try {
      await Neutralino.extensions.dispatch(
        'com.github.vantoan1511.tableview.db-bridge',
        'dbBridge.log',
        {
          level,
          message
        }
      );
    } catch (e) {
      // Fallback to console if extension fails
      console.warn('Failed to send log to extension:', e);
    }
  } else {
    logQueue.push({ level, message });
  }
}

/**
 * Initializes the logger and sets up extension connection monitoring.
 */
export function initLogger() {
  // Monitor extension ready event
  Neutralino.events.on('extensionReady', (event) => {
    if (event.detail === 'com.github.vantoan1511.tableview.db-bridge') {
      isExtensionConnected = true;
      processQueue();
    }
  });

  // Also check if extensions are already loaded
  checkConnection();
}

async function checkConnection() {
  try {
    const stats = await Neutralino.extensions.getStats();
    if (stats.loaded.includes('com.github.vantoan1511.tableview.db-bridge')) {
      isExtensionConnected = true;
      processQueue();
    }
  } catch {
    // Extension might not be ready yet
  }
}

function processQueue() {
  while (logQueue.length > 0) {
    const log = logQueue.shift();
    if (log) {
      sendToExtension(log.level, log.message);
    }
  }
}

/**
 * Overrides global console methods to pipe logs to the backend.
 */
export function setupConsoleOverride() {
  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;
  const originalDebug = console.debug;

  console.log = (...args: any[]) => {
    originalLog(...args);
    sendToExtension('info', formatArgs(args));
  };

  console.warn = (...args: any[]) => {
    originalWarn(...args);
    sendToExtension('warn', formatArgs(args));
  };

  console.error = (...args: any[]) => {
    originalError(...args);
    sendToExtension('error', formatArgs(args));
  };

  console.debug = (...args: any[]) => {
    originalDebug(...args);
    sendToExtension('debug', formatArgs(args));
  };
}

function formatArgs(args: any[]): string {
  return args
    .map((arg) => {
      if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    })
    .join(' ');
}
