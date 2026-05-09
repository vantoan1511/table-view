// ─── Connection ─────────────────────────────────────────────────────────────

export enum DbType {
  POSTGRESQL = 'postgresql',
  MYSQL = 'mysql',
  SQLITE = 'sqlite',
  ORACLE = 'oracle',
  SQLSERVER = 'sqlserver',
  MARIADB = 'mariadb',
  REDIS = 'redis'
}

export enum OracleConnectType {
  SERVICE_NAME = 'serviceName',
  SID = 'sid'
}

export enum OracleRole {
  NORMAL = 'normal',
  SYSDBA = 'sysdba',
  SYSOPER = 'sysoper'
}

export enum ConnectionColor {
  INDIGO = 'indigo',
  BLUE = 'blue',
  TEAL = 'teal',
  GREEN = 'green',
  AMBER = 'amber',
  ORANGE = 'orange',
  PINK = 'pink',
  GRAY = 'gray'
}

export interface Connection {
  id: string;
  name: string;
  type: DbType;
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  color: ConnectionColor;
  tags?: string;
  savePassword: boolean;
  displayAllDatabases: boolean;
  oracleConnectType?: OracleConnectType;
  oracleRole?: OracleRole;
  isConnected: boolean;
}

// ─── Tabs ───────────────────────────────────────────────────────────────────

export enum TabType {
  TABLE = 'table',
  SQL = 'sql'
}

export interface Tab {
  id: string;
  type: TabType;
  title: string;
  tableName?: string;
  query?: string;
  scrollPosition?: number;
  minimized?: boolean;
  connectionId?: string;
  schema?: string;
  dbName?: string;
  isDraft?: boolean;
  isDirty?: boolean;
  filePath?: string;
  closed?: boolean;
}

// ─── Schema ─────────────────────────────────────────────────────────────────

export interface SchemaTable {
  name: string;
  schema: string;
  rowCount?: number;
}

export interface SchemaView {
  name: string;
  schema: string;
}

export interface SchemaFunction {
  name: string;
  schema: string;
  returnType?: string;
}

export interface SchemaInfo {
  tables: SchemaTable[];
  views: SchemaView[];
  functions: SchemaFunction[];
  schemas: string[];
  /** Populated when allDatabases=true. Lists all database names on the server. */
  databases?: string[];
}

// ─── Grid ───────────────────────────────────────────────────────────────────

export interface GridColumn {
  name: string;
  dataType: string;
  isPrimaryKey?: boolean;
  isNullable?: boolean;
}

export type CellValue = string | number | boolean | null;

export interface GridRow {
  [key: string]: CellValue;
}

export interface GridState {
  columns: GridColumn[];
  rows: GridRow[];
  totalRows: number;
  currentPage: number;
  rowsPerPage: number;
  sortColumn?: string;
  sortDirection?: 'asc' | 'desc';
  executionTime?: number;
}

// ─── SQL ────────────────────────────────────────────────────────────────────

export interface QueryResult {
  columns: GridColumn[];
  rows: GridRow[];
  rowCount: number;
  executionTime: number;
}

export interface QueryMessage {
  type: 'info' | 'error' | 'warning';
  text: string;
  timestamp: string;
}

// ─── Layout ─────────────────────────────────────────────────────────────────

export type PanelPosition = 'bottom' | 'right';

export interface PanelTab {
  id: string;
  title: string;
  icon?: string;
  component: string; // Component name or key
}

export interface Panel {
  id: string;
  title: string;
  position: PanelPosition;
  isVisible: boolean;
  isMinimized: boolean;
  size: number; // Height for bottom, Width for right
  activeTabId: string;
  tabs: PanelTab[];
}

export interface LayoutState {
  panels: Record<string, Panel>;
  sidebarWidth: number;
  isSidebarVisible: boolean;
}
