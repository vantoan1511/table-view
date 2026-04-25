const WebSocket = require('ws');
const crypto = require('crypto');
const fs = require('fs');
const process = require('process');

// Obtain required params to start a WS connection from stdIn.
const processInput = JSON.parse(fs.readFileSync(process.stdin.fd, 'utf-8'));
const NL_PORT = processInput.nlPort;
const NL_TOKEN = processInput.nlToken;
const NL_CTOKEN = processInput.nlConnectToken;
const NL_EXTID = processInput.nlExtensionId;

// Parse NeutralinoJS connection details
const authInfo = {
  nlPort: NL_PORT,
  nlToken: NL_TOKEN,
  nlConnectToken: NL_CTOKEN,
  nlExtId: NL_EXTID
};

// Fallback to command line arguments
const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--nl-port') authInfo.nlPort = args[i + 1];
  if (args[i] === '--nl-token') authInfo.nlToken = args[i + 1];
  if (args[i] === '--nl-connect-token') authInfo.nlConnectToken = args[i + 1];
  if (args[i] === '--nl-extension-id') authInfo.nlExtId = args[i + 1];
}

if (!authInfo.nlPort || !authInfo.nlToken || !authInfo.nlExtId) {
  console.error("Missing NeutralinoJS extension connection details.");
  process.exit(1);
}

// Load drivers
const drivers = {
  'postgresql': require('./drivers/postgres.js'),
  'mysql': null, // To be implemented
  'sqlite': null  // To be implemented
};

// Lazy load other drivers when needed to keep startup fast
function getDriver(type) {
  if (type === 'postgres') type = 'postgresql'; // normalization
  if (!drivers[type]) {
    try {
      drivers[type] = require(`./drivers/${type === 'postgresql' ? 'postgres' : type}.js`);
    } catch (err) {
      console.error(`Failed to load driver for ${type}:`, err);
      throw new Error(`Driver for ${type} not found or not implemented.`);
    }
  }
  return drivers[type];
}

let activeDriver = drivers['postgresql']; // Default

const WS_URL = `ws://localhost:${authInfo.nlPort}?extensionId=${authInfo.nlExtId}&connectToken=${authInfo.nlConnectToken}`;
const ws = new WebSocket(WS_URL);

// Helper to send messages back to the Vue app
function broadcast(event, data) {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({
      id: crypto.randomUUID(),
      method: 'app.broadcast',
      accessToken: authInfo.nlToken,
      data: { event, data }
    }));
  }
}

ws.on('open', () => {
  console.log(`Extension ${authInfo.nlExtId} connected to NeutralinoJS.`);
});

ws.on('message', async (messageData) => {
  let message;
  try {
    message = JSON.parse(messageData);
  } catch (err) {
    return;
  }

  // Graceful shutdown when app closes
  if (message.event === 'windowClose') {
    process.exit(0);
  }

  // Handle custom events from Neutralino.extensions.dispatch
  if (message.event && message.event.startsWith('dbBridge.')) {
    const action = message.event.split('.')[1];
    const payload = message.data;

    try {
      if (action === 'testConnection') {
        const { reqId, config } = payload;
        try {
          const driver = getDriver(config.type);
          await driver.connect(config);
          await driver.disconnect();
          broadcast('dbBridge.testConnectionResult', { reqId, success: true });
        } catch (error) {
          broadcast('dbBridge.testConnectionResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'connect') {
        const { reqId, config } = payload;
        try {
          activeDriver = getDriver(config.type);
          await activeDriver.connect(config);
          broadcast('dbBridge.connectResult', { reqId, success: true });
        } catch (error) {
          broadcast('dbBridge.connectResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'getSchema') {
        const { reqId } = payload;
        try {
          const schema = await activeDriver.getSchema();
          broadcast('dbBridge.getSchemaResult', { reqId, success: true, schema });
        } catch (error) {
          broadcast('dbBridge.getSchemaResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'fetchTableData') {
        const { reqId, tableName, limit, offset, sortColumn, sortDirection } = payload;
        try {
          const result = await activeDriver.fetchTableData(tableName, limit, offset, sortColumn, sortDirection);
          broadcast('dbBridge.fetchTableDataResult', { reqId, success: true, ...result });
        } catch (error) {
          broadcast('dbBridge.fetchTableDataResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'executeQuery') {
        const { reqId, sql } = payload;
        try {
          const result = await activeDriver.query(sql);
          broadcast('dbBridge.executeQueryResult', { reqId, success: true, ...result });
        } catch (error) {
          broadcast('dbBridge.executeQueryResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'updateCell') {
        const { reqId, tableName, pkColumn, pkValue, targetColumn, newValue } = payload;
        try {
          await activeDriver.updateCell(tableName, pkColumn, pkValue, targetColumn, newValue);
          broadcast('dbBridge.updateCellResult', { reqId, success: true });
        } catch (error) {
          broadcast('dbBridge.updateCellResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'exportCSV') {
        const { reqId, tableName, exportPath } = payload;
        try {
          await activeDriver.exportToCSV(tableName, exportPath);
          broadcast('dbBridge.exportCSVResult', { reqId, success: true });
        } catch (error) {
          broadcast('dbBridge.exportCSVResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'insertRow') {
        const { reqId, tableName, data } = payload;
        try {
          const newRow = await activeDriver.insertRow(tableName, data || {});
          broadcast('dbBridge.insertRowResult', { reqId, success: true, row: newRow });
        } catch (error) {
          broadcast('dbBridge.insertRowResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      else if (action === 'deleteRows') {
        const { reqId, tableName, pkColumn, pkValues } = payload;
        try {
          await activeDriver.deleteRows(tableName, pkColumn, pkValues);
          broadcast('dbBridge.deleteRowsResult', { reqId, success: true });
        } catch (error) {
          broadcast('dbBridge.deleteRowsResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
    } catch (err) {
      console.error(`Error handling action ${action}:`, err);
    }
  }
});

ws.on('close', () => {
  console.log('NeutralinoJS server disconnected. Exiting...');
  process.exit(0);
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err);
});
