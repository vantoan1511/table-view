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
  fetchTableData: async (tableName, limit = 100, offset = 0) => {
    if (!module.exports.client) {
      throw new Error("Not connected to database");
    }
    
    // WARNING: For production, ensure tableName is properly escaped to prevent SQL injection!
    // Since this is an internal tool and we select tableName from the schema, it is somewhat safe, 
    // but best practice is to quote identifiers.
    const safeTable = `"${tableName.replace(/"/g, '""')}"`;
    
    const query = `SELECT * FROM public.${safeTable} LIMIT $1 OFFSET $2;`;
    const countQuery = `SELECT COUNT(*) as total FROM public.${safeTable};`;
    
    const [dataResult, countResult] = await Promise.all([
      module.exports.client.query(query, [limit, offset]),
      module.exports.client.query(countQuery)
    ]);
    
    return {
      rows: dataResult.rows,
      fields: dataResult.fields.map(f => ({ name: f.name, dataTypeID: f.dataTypeID })),
      totalCount: parseInt(countResult.rows[0].total, 10)
    };
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
