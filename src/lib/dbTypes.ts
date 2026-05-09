import type { DbType } from '@/types';

// ─── Database Type Metadata ───────────────────────────────────────────────────
// Single source of truth for DB type icons, labels, and default ports.
// Used by the connection modal, sidebar tree, and anywhere else DB types are displayed.

export interface DbTypeInfo {
  key: DbType;
  label: string;
  defaultPort: number;
}

export const DB_TYPES: DbTypeInfo[] = [
  { key: 'postgresql', label: 'PostgreSQL', defaultPort: 5432 },
  { key: 'mysql', label: 'MySQL', defaultPort: 3306 },
  { key: 'sqlite', label: 'SQLite', defaultPort: 0 },
  { key: 'oracle', label: 'Oracle', defaultPort: 1521 },
  { key: 'sqlserver', label: 'SQL Server', defaultPort: 1433 },
  { key: 'mariadb', label: 'MariaDB', defaultPort: 3306 },
  { key: 'redis', label: 'Redis', defaultPort: 6379 }
];

/** Quick lookup: DbType -> display label */
export const DB_TYPE_LABEL: Record<string, string> = Object.fromEntries(
  DB_TYPES.map((d) => [d.key, d.label])
);

/** Get the label for a DbType, with a sensible fallback. */
export const getDbTypeLabel = (type: string): string => DB_TYPE_LABEL[type] ?? type;
