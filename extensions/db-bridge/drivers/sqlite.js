const sqlite3 = require('sqlite3');
const fs = require('fs');

module.exports = {
  db: null,

  connect: async (config) => {
    return new Promise((resolve, reject) => {
      // For SQLite, "database" field should contain the file path
      const dbPath = config.database;
      if (!dbPath) {
        return reject(new Error("SQLite requires a database file path."));
      }

      module.exports.db = new sqlite3.Database(dbPath, sqlite3.OPEN_READWRITE | sqlite3.OPEN_CREATE, (err) => {
        if (err) return reject(err);
        resolve(true);
      });
    });
  },

  disconnect: async () => {
    return new Promise((resolve, reject) => {
      if (module.exports.db) {
        module.exports.db.close((err) => {
          if (err) return reject(err);
          module.exports.db = null;
          resolve(true);
        });
      } else {
        resolve(true);
      }
    });
  },

  query: async (sql) => {
    if (!module.exports.db) {
      throw new Error("Not connected to database");
    }
    return new Promise((resolve, reject) => {
      module.exports.db.all(sql, [], (err, rows) => {
        if (err) return reject(err);
        // SQLite doesn't return field metadata like pg/mysql in the same way
        // We'll infer column names from the first row if available
        const fields = rows.length > 0 ? Object.keys(rows[0]).map(name => ({ name, dataTypeID: 0 })) : [];
        resolve({
          rows,
          fields,
          rowCount: rows.length
        });
      });
    });
  },

  getSchema: async () => {
    if (!module.exports.db) {
      throw new Error("Not connected to database");
    }

    const tablesQuery = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
    const viewsQuery = "SELECT name FROM sqlite_master WHERE type='view'";

    const getTables = () => new Promise((res, rej) => module.exports.db.all(tablesQuery, (err, rows) => err ? rej(err) : res(rows)));
    const getViews = () => new Promise((res, rej) => module.exports.db.all(viewsQuery, (err, rows) => err ? rej(err) : res(rows)));

    const [tables, views] = await Promise.all([getTables(), getViews()]);

    return {
      tables: tables.map(t => ({ name: t.name, schema: 'main' })),
      views: views.map(v => ({ name: v.name, schema: 'main' })),
      functions: [] // SQLite doesn't have stored functions in the same way
    };
  },

  fetchTableData: async (tableName, limit = 100, offset = 0, sortColumn = null, sortDirection = 'asc') => {
    if (!module.exports.db) {
      throw new Error("Not connected to database");
    }

    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    
    // Get column info to find PK
    const pragmaQuery = `PRAGMA table_info(${safeTable})`;
    const columns = await new Promise((res, rej) => module.exports.db.all(pragmaQuery, (err, rows) => err ? rej(err) : res(rows)));
    
    const pkColumns = columns.filter(c => c.pk === 1).map(c => c.name);

    let orderClause = '';
    if (sortColumn) {
      const safeCol = `"${sortColumn.replace(/"/g, '""')}"`;
      const dir = sortDirection.toUpperCase() === 'DESC' ? 'DESC' : 'ASC';
      orderClause = ` ORDER BY ${safeCol} ${dir}`;
    }

    const rows = await new Promise((res, rej) => {
      module.exports.db.all(`SELECT * FROM ${safeTable}${orderClause} LIMIT ? OFFSET ?`, [limit, offset], (err, data) => err ? rej(err) : res(data));
    });

    const countResult = await new Promise((res, rej) => {
      module.exports.db.get(`SELECT COUNT(*) as total FROM ${safeTable}`, (err, row) => err ? rej(err) : res(row));
    });

    return {
      rows,
      fields: columns.map(c => ({
        name: c.name,
        dataTypeID: c.type,
        isPrimaryKey: c.pk === 1
      })),
      totalCount: countResult.total
    };
  },

  updateCell: async (tableName, pkColumn, pkValue, targetColumn, newValue) => {
    if (!module.exports.db) {
      throw new Error("Not connected to database");
    }

    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const safeTarget = `"${targetColumn.replace(/"/g, '""')}"`;
    const safePk = `"${pkColumn.replace(/"/g, '""')}"`;

    return new Promise((res, rej) => {
      module.exports.db.run(
        `UPDATE ${safeTable} SET ${safeTarget} = ? WHERE ${safePk} = ?`,
        [newValue, pkValue],
        (err) => err ? rej(err) : res(true)
      );
    });
  },

  insertRow: async (tableName, data) => {
    if (!module.exports.db) throw new Error("Not connected to database");
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const cols = Object.keys(data);
    if (cols.length === 0) {
      return new Promise((res, rej) => {
        module.exports.db.run(`INSERT INTO ${safeTable} DEFAULT VALUES`, [], function(err) {
          if (err) return rej(err);
          module.exports.db.get(`SELECT * FROM ${safeTable} WHERE rowid = ?`, [this.lastID], (e, row) => e ? rej(e) : res(row));
        });
      });
    }
    const safeCols = cols.map(c => `"${c.replace(/"/g, '""')}"`).join(', ');
    const placeholders = cols.map(() => '?').join(', ');
    const values = cols.map(c => data[c]);
    return new Promise((res, rej) => {
      module.exports.db.run(
        `INSERT INTO ${safeTable} (${safeCols}) VALUES (${placeholders})`, values,
        function(err) {
          if (err) return rej(err);
          module.exports.db.get(`SELECT * FROM ${safeTable} WHERE rowid = ?`, [this.lastID], (e, row) => e ? rej(e) : res(row));
        }
      );
    });
  },

  deleteRows: async (tableName, pkColumn, pkValues) => {
    if (!module.exports.db) throw new Error("Not connected to database");
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const safePk = `"${pkColumn.replace(/"/g, '""')}"`;
    const placeholders = pkValues.map(() => '?').join(', ');
    return new Promise((res, rej) => {
      module.exports.db.run(
        `DELETE FROM ${safeTable} WHERE ${safePk} IN (${placeholders})`, pkValues,
        (err) => err ? rej(err) : res(true)
      );
    });
  },

  exportToCSV: async (tableName, exportPath) => {
    if (!module.exports.db) {
      throw new Error("Not connected to database");
    }

    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const writeStream = fs.createWriteStream(exportPath);

    // Get headers
    const columns = await new Promise((res, rej) => module.exports.db.all(`PRAGMA table_info(${safeTable})`, (err, rows) => err ? rej(err) : res(rows)));
    const headers = columns.map(c => c.name);
    writeStream.write(headers.join(',') + '\n');

    let offset = 0;
    const batchSize = 2000;
    let hasMore = true;

    while (hasMore) {
      const rows = await new Promise((res, rej) => {
        module.exports.db.all(`SELECT * FROM ${safeTable} LIMIT ? OFFSET ?`, [batchSize, offset], (err, data) => err ? rej(err) : res(data));
      });

      if (rows.length === 0) {
        hasMore = false;
        break;
      }

      for (const row of rows) {
        const line = headers.map(h => {
          let val = row[h];
          if (val === null || val === undefined) return '';
          val = String(val).replace(/"/g, '""');
          if (val.includes(',') || val.includes('\n') || val.includes('"')) {
            return `"${val}"`;
          }
          return val;
        }).join(',');
        writeStream.write(line + '\n');
      }
      offset += batchSize;
    }

    return new Promise((resolve, reject) => {
      writeStream.end(() => resolve(true));
      writeStream.on('error', reject);
    });
  },

  getTableColumns: async (tableName) => {
    if (!module.exports.db) throw new Error("Not connected to database");
    const columns = await new Promise((res, rej) => {
      module.exports.db.all(`PRAGMA table_info("${tableName.replace(/"/g, '""')}")`, (err, data) => err ? rej(err) : res(data));
    });
    return columns.map(c => ({
      name: c.name,
      dataType: c.type,
      nullable: c.notnull === 0,
      default: c.dflt_value
    }));
  },

  alterTable: async (tableName, operations) => {
    if (!module.exports.db) throw new Error("Not connected to database");
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    for (const op of operations) {
      if (op.type === 'ADD_COLUMN') {
        const safeCol = `"${op.name.replace(/"/g, '""')}"`;
        let def = `ADD COLUMN ${safeCol} ${op.dataType}`;
        if (op.nullable === false) def += ' NOT NULL';
        if (op.default !== undefined && op.default !== null && op.default !== '') {
          const isNumericOrFunc = /^[0-9]+$/.test(op.default) || op.default.includes('(');
          def += ` DEFAULT ${isNumericOrFunc ? op.default : "'" + op.default.replace(/'/g, "''") + "'"}`;
        }
        await new Promise((res, rej) => {
          module.exports.db.run(`ALTER TABLE ${safeTable} ${def}`, (err) => err ? rej(err) : res());
        });
      }
      else if (op.type === 'DROP_COLUMN') {
        const safeCol = `"${op.name.replace(/"/g, '""')}"`;
        await new Promise((res, rej) => {
          module.exports.db.run(`ALTER TABLE ${safeTable} DROP COLUMN ${safeCol}`, (err) => err ? rej(err) : res());
        });
      }
      else if (op.type === 'RENAME_COLUMN') {
        const safeOld = `"${op.oldName.replace(/"/g, '""')}"`;
        const safeNew = `"${op.newName.replace(/"/g, '""')}"`;
        await new Promise((res, rej) => {
          module.exports.db.run(`ALTER TABLE ${safeTable} RENAME COLUMN ${safeOld} TO ${safeNew}`, (err) => err ? rej(err) : res());
        });
      }
    }
    return true;
  }
};
