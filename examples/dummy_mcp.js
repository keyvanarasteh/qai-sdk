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
    } else if (req.method === 'prompts/list') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          prompts: [
            {
              name: 'code_review',
              description: 'Review the provided code string',
              arguments: [
                {
                  name: 'code',
                  description: 'The code to review',
                  required: true
                }
              ]
            }
          ]
        }
      }));
    } else if (req.method === 'prompts/get') {
      const args = req.params?.arguments || {};
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          description: 'Response to code review',
          messages: [
            {
              role: 'user',
              content: {
                type: 'text',
                text: `Please review the following code for security vulnerabilities:\n\n${args.code || ''}`
              }
            }
          ]
        }
      }));
    } else if (req.method === 'resources/list') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          resources: [
            {
              uri: 'file:///logs/app.log',
              name: 'Application Logs',
              description: 'The main application log file',
              mimeType: 'text/plain'
            }
          ]
        }
      }));
    } else if (req.method === 'resources/templates/list') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          resourceTemplates: [
            {
              uriTemplate: 'file:///logs/{date}.log',
              name: 'Archived Logs',
              description: 'Access logs for a specific date',
              mimeType: 'text/plain'
            }
          ]
        }
      }));
    } else if (req.method === 'resources/read') {
      const uri = req.params?.uri || '';
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: req.id,
        result: {
          contents: [
            {
              uri: uri,
              mimeType: 'text/plain',
              text: `[DEBUG] Mock resource content for ${uri}\n[INFO] Server timestamp: ${new Date().toISOString()}`
            }
          ]
        }
      }));
    } else if (req.method === 'resources/subscribe') {
      const uri = req.params?.uri || '';
      console.log(JSON.stringify({ jsonrpc: '2.0', id: req.id, result: {} }));
      
      if (!global.resourceIntervals) global.resourceIntervals = {};
      if (global.resourceIntervals[uri]) clearInterval(global.resourceIntervals[uri]);
      
      global.resourceIntervals[uri] = setInterval(() => {
        console.log(JSON.stringify({
          jsonrpc: '2.0',
          method: 'notifications/resources/updated',
          params: { uri }
        }));
      }, 500);
      
    } else if (req.method === 'resources/unsubscribe') {
      const uri = req.params?.uri || '';
      if (global.resourceIntervals && global.resourceIntervals[uri]) {
          clearInterval(global.resourceIntervals[uri]);
          delete global.resourceIntervals[uri];
      }
      console.log(JSON.stringify({ jsonrpc: '2.0', id: req.id, result: {} }));
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
