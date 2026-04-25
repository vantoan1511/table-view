const { Client } = require('pg');

module.exports = {
  // Store the active client instance
  client: null,

  /**
   * Connect to the PostgreSQL database
   * @param {Object} config Connection config (host, port, user, password, database)
   */
  connect: async (config) => {
    // If there's an existing client, disconnect it first
    if (module.exports.client) {
      await module.exports.disconnect();
    }

    const client = new Client({
      host: config.host || 'localhost',
      port: config.port ? parseInt(config.port, 10) : 5432,
      user: config.user || config.username,
      password: config.password,
      database: config.database,
      ssl: config.ssl ? { rejectUnauthorized: false } : false,
      connectionTimeoutMillis: 5000 // 5 seconds timeout
    });

    await client.connect();
    module.exports.client = client;
    return true;
  },

  /**
   * Execute an arbitrary SQL query
   * @param {string} sql The SQL query string
   */
  query: async (sql) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    
    const result = await module.exports.client.query(sql);
    return {
      rows: result.rows,
      fields: result.fields.map(f => ({ name: f.name, dataTypeID: f.dataTypeID })),
      rowCount: result.rowCount
    };
  },

  /**
   * Get the database schema (tables, views, functions)
   */
  getSchema: async () => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }

    // Tables
    const tablesResult = await module.exports.client.query(`
      SELECT table_name 
      FROM information_schema.tables 
      WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
      ORDER BY table_name;
    `);

    // Views
    const viewsResult = await module.exports.client.query(`
      SELECT table_name 
      FROM information_schema.views 
      WHERE table_schema = 'public'
      ORDER BY table_name;
    `);

    // Functions
    const functionsResult = await module.exports.client.query(`
      SELECT routine_name 
      FROM information_schema.routines 
      WHERE routine_schema = 'public' AND routine_type = 'FUNCTION'
      ORDER BY routine_name;
    `);

    return {
      tables: tablesResult.rows.map(r => ({ name: r.table_name })),
      views: viewsResult.rows.map(r => ({ name: r.table_name })),
      functions: functionsResult.rows.map(r => ({ name: r.routine_name })),
      schemas: [{ name: 'public' }]
    };
  },

  /**
   * Fetch data from a specific table with pagination
   */
  fetchTableData: async (tableName, limit = 100, offset = 0, sortColumn = null, sortDirection = 'asc', filter = null) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    
    // Build WHERE clause if filter is specified
    let whereClause = '';
    const queryParams = [limit, offset];
    if (filter) {
      whereClause = ` WHERE CAST(t.* AS TEXT) ILIKE $3`;
      queryParams.push(`%${filter}%`);
    }
    
    // Find primary keys for the table
    const pkQuery = `
      SELECT kcu.column_name
      FROM information_schema.table_constraints tco
      JOIN information_schema.key_column_usage kcu 
           ON kcu.constraint_name = tco.constraint_name
           AND kcu.constraint_schema = tco.constraint_schema
      WHERE tco.constraint_type = 'PRIMARY KEY' 
        AND kcu.table_name = $1
        AND tco.table_schema = 'public';
    `;
    
    // Build ORDER BY clause if sort is specified
    let orderClause = '';
    if (sortColumn) {
      const safeCol = `"${sortColumn.replace(/"/g, '""')}"`;
      const dir = sortDirection === 'desc' ? 'DESC' : 'ASC';
      orderClause = ` ORDER BY ${safeCol} ${dir}`;
    }
    
    const query = `SELECT * FROM public.${safeTable} t${whereClause}${orderClause} LIMIT $1 OFFSET $2;`;
    const countQuery = `SELECT COUNT(*) as total FROM public.${safeTable} t${whereClause};`;
    
    const pkResult = await module.exports.client.query(pkQuery, [tableName]);
    const dataResult = await module.exports.client.query(query, queryParams);
    
    // For count query, if there's a filter, it will be the third param in whereClause ($3)
    // but the count query doesn't have $1 and $2 (limit/offset).
    // So we replace $3 with $1 in the count query.
    const finalCountQuery = filter ? countQuery.replace('$3', '$1') : countQuery;
    const finalCountParams = filter ? [queryParams[2]] : [];
    const countResult = await module.exports.client.query(finalCountQuery, finalCountParams);
    
    const pkColumns = pkResult.rows.map(r => r.column_name);

    return {
      rows: dataResult.rows,
      fields: dataResult.fields.map(f => ({ 
        name: f.name, 
        dataTypeID: f.dataTypeID,
        isPrimaryKey: pkColumns.includes(f.name)
      })),
      totalCount: parseInt(countResult.rows[0].total, 10)
    };
  },

  /**
   * Update a specific cell in a table
   */
  updateCell: async (tableName, pkColumn, pkValue, targetColumn, newValue) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const safePkCol = `"${pkColumn.replace(/"/g, '""')}"`;
    const safeTargetCol = `"${targetColumn.replace(/"/g, '""')}"`;
    
    const query = `UPDATE public.${safeTable} SET ${safeTargetCol} = $1 WHERE ${safePkCol} = $2`;
    await module.exports.client.query(query, [newValue, pkValue]);
    return true;
  },

  /**
   * Insert a new row into a table
   * @param {string} tableName
   * @param {Object} data  { colName: value, … }
   * Returns the inserted row (with DB-generated defaults like serial IDs).
   */
  insertRow: async (tableName, data) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const cols = Object.keys(data);
    if (cols.length === 0) {
      // INSERT with all defaults
      const result = await module.exports.client.query(
        `INSERT INTO public.${safeTable} DEFAULT VALUES RETURNING *`
      );
      return result.rows[0];
    }
    const safeCols = cols.map(c => `"${c.replace(/"/g, '""')}"`).join(', ');
    const placeholders = cols.map((_, i) => `$${i + 1}`).join(', ');
    const values = cols.map(c => data[c]);
    
    const query = `INSERT INTO public.${safeTable} (${safeCols}) VALUES (${placeholders}) RETURNING *`;
    console.log('[Postgres Driver] insertRow Executing:', query, 'with values:', values);
    
    try {
      const result = await module.exports.client.query(query, values);
      return result.rows[0];
    } catch (err) {
      throw new Error(`[Query: ${query} | Values: ${JSON.stringify(values)}] ${err.message}`);
    }
  },

  /**
   * Delete rows from a table by primary key values
   * @param {string} tableName
   * @param {string} pkColumn
   * @param {Array}  pkValues  Array of PK values to delete
   */
  deleteRows: async (tableName, pkColumn, pkValues) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    const safePk = `"${pkColumn.replace(/"/g, '""')}"`;
    const placeholders = pkValues.map((_, i) => `$${i + 1}`).join(', ');
    await module.exports.client.query(
      `DELETE FROM public.${safeTable} WHERE ${safePk} = ANY(ARRAY[${placeholders}])`,
      pkValues
    );
    return true;
  },

  /**
   * Export table data to CSV file
   */
  exportToCSV: async (tableName, exportPath) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    const fs = require('fs');
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    
    const writeStream = fs.createWriteStream(exportPath);
    
    // Write header
    const schemaResult = await module.exports.client.query(`SELECT * FROM public.${safeTable} LIMIT 0`);
    const headers = schemaResult.fields.map(f => f.name);
    writeStream.write(headers.join(',') + '\n');
    
    // Simple batching to avoid memory issues on large tables
    let offset = 0;
    const batchSize = 2000;
    let hasMore = true;
    
    while (hasMore) {
      const result = await module.exports.client.query(`SELECT * FROM public.${safeTable} LIMIT $1 OFFSET $2`, [batchSize, offset]);
      if (result.rows.length === 0) {
        hasMore = false;
        break;
      }
      
      for (const row of result.rows) {
        const line = headers.map(h => {
          let val = row[h];
          if (val === null || val === undefined) return '';
          val = String(val).replace(/"/g, '""');
          if (val.includes(',') || val.includes('\\n') || val.includes('"')) {
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

  /**
   * Get detailed column definitions for a table
   */
  getTableColumns: async (tableName) => {
    if (!module.exports.client) throw new Error("Not connected to database");
    
    // Fetch columns with metadata
    const query = `
      SELECT 
        c.column_name as name,
        c.data_type as "dataType",
        c.is_nullable as nullable,
        c.column_default as "default"
      FROM information_schema.columns c
      WHERE c.table_schema = 'public' AND c.table_name = $1
      ORDER BY c.ordinal_position;
    `;
    const result = await module.exports.client.query(query, [tableName]);
    return result.rows.map(r => ({
      name: r.name,
      dataType: r.dataType,
      nullable: r.nullable === 'YES',
      default: r.default
    }));
  },

  /**
   * Alter table (ADD, DROP, RENAME columns)
   */
  alterTable: async (tableName, operations) => {
    if (!module.exports.client) throw new Error("Not connected to database");
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;

    // Process each operation sequentially
    for (const op of operations) {
      if (op.type === 'ADD_COLUMN') {
        const safeCol = `"${op.name.replace(/"/g, '""')}"`;
        let def = `ADD COLUMN ${safeCol} ${op.dataType}`;
        if (op.nullable === false) def += ' NOT NULL';
        if (op.default !== undefined && op.default !== null && op.default !== '') {
          // simple check for numbers/functions vs strings
          const isNumericOrFunc = /^[0-9]+$/.test(op.default) || op.default.includes('(');
          def += ` DEFAULT ${isNumericOrFunc ? op.default : "'" + op.default.replace(/'/g, "''") + "'"}`;
        }
        await module.exports.client.query(`ALTER TABLE public.${safeTable} ${def}`);
      } 
      else if (op.type === 'DROP_COLUMN') {
        const safeCol = `"${op.name.replace(/"/g, '""')}"`;
        await module.exports.client.query(`ALTER TABLE public.${safeTable} DROP COLUMN ${safeCol}`);
      } 
      else if (op.type === 'RENAME_COLUMN') {
        const safeOld = `"${op.oldName.replace(/"/g, '""')}"`;
        const safeNew = `"${op.newName.replace(/"/g, '""')}"`;
        await module.exports.client.query(`ALTER TABLE public.${safeTable} RENAME COLUMN ${safeOld} TO ${safeNew}`);
      }
    }
    return true;
  },

  /**
   * Disconnect from the database
   */
  disconnect: async () => {
    if (module.exports.client) {
      try {
        await module.exports.client.end();
      } catch (err) {
        console.error("Error disconnecting from postgres:", err);
      } finally {
        module.exports.client = null;
      }
    }
  }
};
