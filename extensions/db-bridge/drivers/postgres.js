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
  fetchTableData: async (tableName, limit = 100, offset = 0, sortColumn = null, sortDirection = 'asc') => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    
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
    
    const query = `SELECT * FROM public.${safeTable}${orderClause} LIMIT $1 OFFSET $2;`;
    const countQuery = `SELECT COUNT(*) as total FROM public.${safeTable};`;
    
    const [pkResult, dataResult, countResult] = await Promise.all([
      module.exports.client.query(pkQuery, [tableName]),
      module.exports.client.query(query, [limit, offset]),
      module.exports.client.query(countQuery)
    ]);
    
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
