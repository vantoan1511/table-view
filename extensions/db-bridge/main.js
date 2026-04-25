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
  'postgres': require('./drivers/postgres.js')
};

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
        
        // TODO: Handle other DB types later, currently defaulting to postgres
        const driver = drivers['postgres'];
        
        try {
          await driver.connect(config);
          await driver.disconnect();
          broadcast('dbBridge.testConnectionResult', { reqId, success: true });
        } catch (error) {
          broadcast('dbBridge.testConnectionResult', { reqId, success: false, error: String(error.message || error) });
        }
      }
      
      // Additional actions like getSchema, executeQuery will go here in future
      
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
