const mysql = require('mysql2/promise');
const fs = require('fs');

module.exports = {
  client: null,

  connect: async (config) => {
    module.exports.client = await mysql.createConnection({
      host: config.host,
      port: config.port,
      user: config.username,
      password: config.password,
      database: config.database,
      connectTimeout: (config.connectionTimeout || 30) * 1000,
    });
    return true;
  },

  disconnect: async () => {
    if (module.exports.client) {
      await module.exports.client.end();
      module.exports.client = null;
    }
  },

  query: async (sql) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    const [rows, fields] = await module.exports.client.execute(sql);
    return {
      rows,
      fields: fields.map(f => ({
        name: f.name,
        dataTypeID: f.columnType
      })),
      rowCount: rows.length
    };
  },

  getSchema: async () => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }

    // Get Tables
    const [tables] = await module.exports.client.execute(`
      SELECT TABLE_NAME as name, 'table' as type
      FROM information_schema.TABLES
      WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE'
    `);

    // Get Views
    const [views] = await module.exports.client.execute(`
      SELECT TABLE_NAME as name, 'view' as type
      FROM information_schema.TABLES
      WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'VIEW'
    `);

    // Get Procedures/Functions
    const [routines] = await module.exports.client.execute(`
      SELECT ROUTINE_NAME as name, ROUTINE_TYPE as type
      FROM information_schema.ROUTINES
      WHERE ROUTINE_SCHEMA = DATABASE()
    `);

    return {
      tables: tables.map(t => ({ name: t.name, schema: 'public' })), // MySQL doesn't have schemas like Postgres, database() is the "schema"
      views: views.map(v => ({ name: v.name, schema: 'public' })),
      functions: routines.map(r => ({ name: r.name, schema: 'public', type: r.type }))
    };
  },

  fetchTableData: async (tableName, limit = 100, offset = 0, sortColumn = null, sortDirection = 'asc') => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }

    const safeTable = `\`${tableName.replace(/`/g, '``')}\``;
    
    // Find primary keys
    const [pkResult] = await module.exports.client.execute(`
      SELECT COLUMN_NAME
      FROM information_schema.COLUMNS
      WHERE TABLE_SCHEMA = DATABASE()
        AND TABLE_NAME = ?
        AND COLUMN_KEY = 'PRI'
    `, [tableName]);
    
    const pkColumns = pkResult.map(r => r.COLUMN_NAME);

    let orderClause = '';
    if (sortColumn) {
      const safeCol = `\`${sortColumn.replace(/`/g, '``')}\``;
      const dir = sortDirection.toUpperCase() === 'DESC' ? 'DESC' : 'ASC';
      orderClause = ` ORDER BY ${safeCol} ${dir}`;
    }

    const [rows, fields] = await module.exports.client.execute(
      `SELECT * FROM ${safeTable}${orderClause} LIMIT ? OFFSET ?`,
      [limit, offset]
    );

    const [countResult] = await module.exports.client.execute(
      `SELECT COUNT(*) as total FROM ${safeTable}`
    );

    return {
      rows,
      fields: fields.map(f => ({
        name: f.name,
        dataTypeID: f.columnType,
        isPrimaryKey: pkColumns.includes(f.name)
      })),
      totalCount: countResult[0].total
    };
  },

  updateCell: async (tableName, pkColumn, pkValue, targetColumn, newValue) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }

    const safeTable = `\`${tableName.replace(/`/g, '``')}\``;
    const safeTarget = `\`${targetColumn.replace(/`/g, '``')}\``;
    const safePk = `\`${pkColumn.replace(/`/g, '``')}\``;

    await module.exports.client.execute(
      `UPDATE ${safeTable} SET ${safeTarget} = ? WHERE ${safePk} = ?`,
      [newValue, pkValue]
    );
  },

  insertRow: async (tableName, data) => {
    if (!module.exports.client) throw new Error("Not connected to database");
    const safeTable = `\`${tableName.replace(/`/g, '``')}\``;
    const cols = Object.keys(data);
    if (cols.length === 0) {
      const [result] = await module.exports.client.execute(`INSERT INTO ${safeTable} () VALUES ()`);
      const [rows] = await module.exports.client.execute(`SELECT * FROM ${safeTable} WHERE id = ?`, [result.insertId]);
      return rows[0];
    }
    const safeCols = cols.map(c => `\`${c.replace(/`/g, '``')}\``).join(', ');
    const placeholders = cols.map(() => '?').join(', ');
    const values = cols.map(c => data[c]);
    const [result] = await module.exports.client.execute(
      `INSERT INTO ${safeTable} (${safeCols}) VALUES (${placeholders})`, values
    );
    const [rows] = await module.exports.client.execute(`SELECT * FROM ${safeTable} WHERE id = ?`, [result.insertId]);
    return rows[0] || { insertId: result.insertId };
  },

  deleteRows: async (tableName, pkColumn, pkValues) => {
    if (!module.exports.client) throw new Error("Not connected to database");
    const safeTable = `\`${tableName.replace(/`/g, '``')}\``;
    const safePk = `\`${pkColumn.replace(/`/g, '``')}\``;
    const placeholders = pkValues.map(() => '?').join(', ');
    await module.exports.client.execute(
      `DELETE FROM ${safeTable} WHERE ${safePk} IN (${placeholders})`, pkValues
    );
    return true;
  },

  exportToCSV: async (tableName, exportPath) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }

    const safeTable = `\`${tableName.replace(/`/g, '``')}\``;
    const writeStream = fs.createWriteStream(exportPath);

    const [fieldsResult] = await module.exports.client.execute(`SELECT * FROM ${safeTable} LIMIT 0`);
    const headers = fieldsResult.map(f => f.name);
    writeStream.write(headers.join(',') + '\n');

    let offset = 0;
    const batchSize = 2000;
    let hasMore = true;

    while (hasMore) {
      const [rows] = await module.exports.client.execute(
        `SELECT * FROM ${safeTable} LIMIT ? OFFSET ?`,
        [batchSize, offset]
      );

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
    if (!module.exports.connection) throw new Error("Not connected to database");
    const [rows] = await module.exports.connection.query(`
      SELECT 
        COLUMN_NAME as name, 
        DATA_TYPE as dataType, 
        IS_NULLABLE as isNullable, 
        COLUMN_DEFAULT as columnDefault 
      FROM INFORMATION_SCHEMA.COLUMNS 
      WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
      ORDER BY ORDINAL_POSITION
    `, [tableName]);
    return rows.map(r => ({
      name: r.name,
      dataType: r.dataType,
      nullable: r.isNullable === 'YES',
      default: r.columnDefault
    }));
  },

  alterTable: async (tableName, operations) => {
    if (!module.exports.connection) throw new Error("Not connected to database");
    const safeTable = `\`${tableName.replace(/`/g, '``')}\``;
    for (const op of operations) {
      if (op.type === 'ADD_COLUMN') {
        const safeCol = `\`${op.name.replace(/`/g, '``')}\``;
        let def = `ADD COLUMN ${safeCol} ${op.dataType}`;
        if (op.nullable === false) def += ' NOT NULL';
        if (op.default !== undefined && op.default !== null && op.default !== '') {
          const isNumericOrFunc = /^[0-9]+$/.test(op.default) || op.default.includes('(');
          def += ` DEFAULT ${isNumericOrFunc ? op.default : "'" + op.default.replace(/'/g, "''") + "'"}`;
        }
        await module.exports.connection.query(`ALTER TABLE ${safeTable} ${def}`);
      }
      else if (op.type === 'DROP_COLUMN') {
        const safeCol = `\`${op.name.replace(/`/g, '``')}\``;
        await module.exports.connection.query(`ALTER TABLE ${safeTable} DROP COLUMN ${safeCol}`);
      }
      else if (op.type === 'RENAME_COLUMN') {
        const safeOld = `\`${op.oldName.replace(/`/g, '``')}\``;
        const safeNew = `\`${op.newName.replace(/`/g, '``')}\``;
        await module.exports.connection.query(`ALTER TABLE ${safeTable} RENAME COLUMN ${safeOld} TO ${safeNew}`);
      }
    }
    return true;
  }
};
