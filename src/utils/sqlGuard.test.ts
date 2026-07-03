import { describe, it, expect } from 'vitest';
import { isDestructiveQuery, stripSqlComments } from './sqlGuard';

describe('SQL Guard Utility', () => {
  describe('stripSqlComments', () => {
    it('strips single-line comments', () => {
      const sql = 'SELECT * FROM users; -- this is a comment';
      expect(stripSqlComments(sql).trim()).toBe('SELECT * FROM users;');
    });

    it('strips multi-line comments', () => {
      const sql = 'SELECT * /* multi-line\ncomment */ FROM users;';
      expect(stripSqlComments(sql).trim()).toBe('SELECT *  FROM users;');
    });
  });

  describe('isDestructiveQuery', () => {
    it('returns false for empty query', () => {
      expect(isDestructiveQuery('')).toBe(false);
      expect(isDestructiveQuery('   ')).toBe(false);
    });

    it('returns false for safe SELECT queries', () => {
      expect(isDestructiveQuery('SELECT * FROM users;')).toBe(false);
      expect(isDestructiveQuery('SELECT * FROM users WHERE id = 1;')).toBe(false);
    });

    it('returns true for DROP statements', () => {
      expect(isDestructiveQuery('DROP TABLE users;')).toBe(true);
      expect(isDestructiveQuery('drop database prod;')).toBe(true);
      expect(isDestructiveQuery('DROP view active_users;')).toBe(true);
    });

    it('returns true for TRUNCATE statements', () => {
      expect(isDestructiveQuery('TRUNCATE TABLE logs;')).toBe(true);
      expect(isDestructiveQuery('truncate logs;')).toBe(true);
    });

    it('returns true for UPDATE without WHERE', () => {
      expect(isDestructiveQuery('UPDATE users SET active = false;')).toBe(true);
      expect(isDestructiveQuery('update users set active = false')).toBe(true);
    });

    it('returns false for UPDATE with WHERE', () => {
      expect(isDestructiveQuery('UPDATE users SET active = false WHERE id = 5;')).toBe(false);
      expect(isDestructiveQuery('update users set active = false where id = 5')).toBe(false);
    });

    it('returns true for DELETE without WHERE', () => {
      expect(isDestructiveQuery('DELETE FROM users;')).toBe(true);
      expect(isDestructiveQuery('delete from users')).toBe(true);
    });

    it('returns false for DELETE with WHERE', () => {
      expect(isDestructiveQuery('DELETE FROM users WHERE id = 5;')).toBe(false);
      expect(isDestructiveQuery('delete from users where id = 5')).toBe(false);
    });

    it('handles query comments and correctly identifies destructive commands inside multi-statement inputs', () => {
      // Safe statement followed by destructive one
      expect(isDestructiveQuery('SELECT * FROM users; DELETE FROM users;')).toBe(true);

      // Commented out UPDATE/DELETE without WHERE should be safe
      expect(isDestructiveQuery('-- UPDATE users SET active = false;\nSELECT 1;')).toBe(false);
      expect(isDestructiveQuery('/* DELETE FROM users; */ SELECT 1;')).toBe(false);

      // UPDATE without WHERE but with comments around should still be flagged
      expect(isDestructiveQuery('UPDATE users SET active = false; -- update all users')).toBe(true);
    });

    it('correctly handles where word/boundaries in string literals', () => {
      // "where" inside string literal - should be flagged as destructive (no actual WHERE clause)
      expect(isDestructiveQuery("UPDATE users SET bio = 'I live where it is quiet';")).toBe(true);
      expect(isDestructiveQuery("UPDATE users SET name = 'where';")).toBe(true);
      expect(isDestructiveQuery('UPDATE users SET name = "where";')).toBe(true);

      // "where" inside string literal + actual WHERE clause - should be safe
      expect(
        isDestructiveQuery("UPDATE users SET bio = 'I live where it is quiet' WHERE id = 1;")
      ).toBe(false);
      expect(isDestructiveQuery("UPDATE users SET name = 'where' WHERE id = 1;")).toBe(false);
    });
  });
});
