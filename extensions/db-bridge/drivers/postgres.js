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

    // Simplified query to get tables in the public schema
    // In a real implementation, this would be more comprehensive
    const tablesQuery = `
      SELECT table_name 
      FROM information_schema.tables 
      WHERE table_schema = 'public' 
      ORDER BY table_name;
    `;
    
    const result = await module.exports.client.query(tablesQuery);
    return {
      tables: result.rows.map(r => ({ name: r.table_name })),
      views: [],
      functions: [],
      schemas: [{ name: 'public' }]
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
