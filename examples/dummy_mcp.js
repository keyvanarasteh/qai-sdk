const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

rl.on('line', (line) => {
  if (!line.trim()) return;
  try {
    const req = JSON.parse(line);
    
    // Ignore notifications like initialized
    if (!req.id) return;
    
    if (req.method === 'initialize') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          protocolVersion: '2024-11-05',
          capabilities: {},
          serverInfo: { name: 'dummy-mcp', version: '1.0.0' }
        }
      }));
    } else if (req.method === 'tools/list') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          tools: [
            {
              name: 'get_weather',
              description: 'Get weather for a location',
              inputSchema: {
                type: 'object',
                properties: { location: { type: 'string' } }
              }
            }
          ]
        }
      }));
    } else if (req.method === 'tools/call') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          content: [{ type: 'text', text: 'Weather is sunny' }]
        }
      }));
    } else {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        error: { code: -32601, message: 'Method not found' }
      }));
    }
  } catch (err) {
    // ignore parse errors in dummy
  }
});
