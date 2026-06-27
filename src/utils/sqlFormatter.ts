import { format } from 'sql-formatter';

import { DbType } from '@/types';

export const getFormatterDialect = (dbType?: DbType): string => {
  if (!dbType) return 'sql';

  switch (dbType) {
    case DbType.POSTGRESQL:
      return 'postgresql';
    case DbType.MYSQL:
      return 'mysql';
    case DbType.SQLITE:
      return 'sqlite';
    case DbType.ORACLE:
      return 'plsql';
    case DbType.SQLSERVER:
      return 'tsql';
    case DbType.MARIADB:
      return 'mariadb';
    default:
      return 'sql';
  }
};

export const formatSql = (sql: string, dbType?: DbType): string => {
  if (!sql || sql.trim() === '') return '';

  const dialect = getFormatterDialect(dbType);

  try {
    return format(sql, {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      language: dialect as any,
      keywordCase: 'upper'
    });
  } catch (error) {
    // eslint-disable-next-line no-console
    console.warn(`SQL formatting failed for dialect ${dialect}:`, error);
    return sql;
  }
};
