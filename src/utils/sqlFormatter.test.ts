import { describe, it, expect, vi } from 'vitest';

import { DbType } from '@/types';
import { getFormatterDialect, formatSql } from './sqlFormatter';

describe('SQL Formatter Utility', () => {
  describe('getFormatterDialect', () => {
    it('maps POSTGRESQL to postgresql', () => {
      expect(getFormatterDialect(DbType.POSTGRESQL)).toBe('postgresql');
    });

    it('maps MYSQL to mysql', () => {
      expect(getFormatterDialect(DbType.MYSQL)).toBe('mysql');
    });

    it('maps SQLITE to sqlite', () => {
      expect(getFormatterDialect(DbType.SQLITE)).toBe('sqlite');
    });

    it('maps ORACLE to plsql', () => {
      expect(getFormatterDialect(DbType.ORACLE)).toBe('plsql');
    });

    it('maps SQLSERVER to tsql', () => {
      expect(getFormatterDialect(DbType.SQLSERVER)).toBe('tsql');
    });

    it('maps MARIADB to mariadb', () => {
      expect(getFormatterDialect(DbType.MARIADB)).toBe('mariadb');
    });

    it('maps undefined to sql', () => {
      expect(getFormatterDialect(undefined)).toBe('sql');
    });

    it('maps unsupported DbType to sql', () => {
      expect(getFormatterDialect(DbType.REDIS)).toBe('sql');
    });
  });

  describe('formatSql', () => {
    it('formats a simple query with uppercase keywords', async () => {
      const sql = 'select * from users where id = 1';
      const expected = 'SELECT\n  *\nFROM\n  users\nWHERE\n  id = 1';
      expect(await formatSql(sql, DbType.POSTGRESQL)).toBe(expected);
    });

    it('returns empty string for empty input', async () => {
      expect(await formatSql('')).toBe('');
      expect(await formatSql('   ')).toBe('');
    });

    it('gracefully returns the original sql query on syntax error', async () => {
      const invalidSql = 'select * from (unclosed query';
      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      expect(await formatSql(invalidSql, DbType.POSTGRESQL)).toBe(invalidSql);
      expect(warnSpy).toHaveBeenCalled();
      warnSpy.mockRestore();
    });
  });
});
