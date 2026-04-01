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
