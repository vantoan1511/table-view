export interface TableViewEventMap {
  windowClose: undefined | Record<string, never>;
  extensionReady: string;
  spawnedProcess: {
    id: number;
    action: 'stdOut' | 'stdErr' | 'exit';
    data?: unknown;
  };
  'dbBridge.connectResult': {
    reqId: string;
    success: boolean;
    data?: unknown;
    error?: string;
  };
  'dbBridge.disconnectResult': {
    reqId: string;
    success: boolean;
    data?: unknown;
    error?: string;
  };
  'dbBridge.queryResult': {
    reqId: string;
    success: boolean;
    data?: unknown;
    error?: string;
  };
  'dbBridge.schemaResult': {
    reqId: string;
    success: boolean;
    data?: unknown;
    error?: string;
  };
  'dbBridge.testConnectionResult': {
    reqId: string;
    success: boolean;
    data?: unknown;
    error?: string;
  };
  'dbBridge.exportCSVResult': {
    reqId: string;
    success: boolean;
    data?: unknown;
    error?: string;
  };
  'dbBridge.log': {
    level: string;
    message: string;
  };
  'dbBridge.shutdown': Record<string, never>;
  [key: string]: unknown;
}

export type TableViewEventName = Extract<keyof TableViewEventMap, string>;
