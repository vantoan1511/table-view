/**
 * Strips SQL comments (both single line -- and multi-line /* * /)
 */
export const stripSqlComments = (sql: string): string => {
  return sql
    .replace(/--.*$/gm, '') // Remove single-line comments
    .replace(/\/\*[\s\S]*?\*\//g, ''); // Remove multi-line comments
};

/**
 * Strips the content of single-quoted and double-quoted string literals,
 * replacing their inner characters with spaces.
 */
export const stripSqlStringLiterals = (sql: string): string => {
  let result = '';
  let inSingleQuote = false;
  let inDoubleQuote = false;
  let escapeNext = false;

  for (let i = 0; i < sql.length; i++) {
    const char = sql[i];

    if (escapeNext) {
      result += ' ';
      escapeNext = false;
      continue;
    }

    if (char === '\\') {
      escapeNext = true;
      result += ' ';
      continue;
    }

    if (char === "'" && !inDoubleQuote) {
      inSingleQuote = !inSingleQuote;
      result += "'";
      continue;
    }

    if (char === '"' && !inSingleQuote) {
      inDoubleQuote = !inDoubleQuote;
      result += '"';
      continue;
    }

    if (inSingleQuote || inDoubleQuote) {
      result += ' ';
    } else {
      result += char;
    }
  }
  return result;
};

/**
 * Checks if a SQL query block contains any potentially destructive operations
 * that should trigger a safety warning.
 * Destructive operations include:
 * - DROP (tables, databases, schemas, views, etc.)
 * - TRUNCATE
 * - UPDATE without a WHERE clause
 * - DELETE without a WHERE clause
 */
export const isDestructiveQuery = (sql: string): boolean => {
  if (!sql) return false;

  const cleanSql = stripSqlStringLiterals(stripSqlComments(sql));
  // Split queries by semicolon to check each query statement independently
  const statements = cleanSql
    .split(';')
    .map((s) => s.trim())
    .filter(Boolean);

  for (const statement of statements) {
    const lower = statement.toLowerCase();

    // Check for DROP or TRUNCATE
    if (/\bdrop\b/.test(lower) || /\btruncate\b/.test(lower)) {
      return true;
    }

    // Check for UPDATE or DELETE
    const isUpdate = lower.startsWith('update');
    const isDelete = lower.startsWith('delete');

    if (isUpdate || isDelete) {
      // Check if it has a WHERE clause.
      // We look for the word "where" as a whole word boundary.
      // To be safe, we check if \bwhere\b exists in the statement.
      const hasWhere = /\bwhere\b/i.test(lower);
      if (!hasWhere) {
        return true;
      }
    }
  }

  return false;
};
