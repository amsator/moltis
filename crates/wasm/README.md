# moltis-wasm

WebAssembly build of moltis for browser environments.

## Overview

This crate provides a WASM-compatible subset of moltis functionality, enabling
agent logic to run directly in the browser while delegating I/O-intensive
operations (tool execution, file storage) to a backend gateway.

## Architecture

```
┌─────────────────────────────────────┐
│  Browser (WASM)                     │
│  - Protocol parsing                 │
│  - Message routing                  │
│  - Session state (in-memory)        │
│  - Provider selection               │
└───────────┬─────────────────────────┘
            │ HTTP/WebSocket
┌───────────▼─────────────────────────┐
│  Backend Gateway                    │
│  - Tool execution                   │
│  - File storage                     │
│  - LLM API calls (optional)         │
└─────────────────────────────────────┘
```

## Building

### Prerequisites

1. Install the Rust WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Install wasm-pack:
   ```bash
   cargo install wasm-pack
   ```

### Build Commands

Build for web (bundler-compatible):
```bash
wasm-pack build crates/wasm --target web
```

Build for Node.js:
```bash
wasm-pack build crates/wasm --target nodejs
```

Build with console panic hook for debugging:
```bash
wasm-pack build crates/wasm --target web -- --features console-panic
```

### Output

The build output will be in `crates/wasm/pkg/` and includes:
- `moltis_wasm.js` - JavaScript bindings
- `moltis_wasm_bg.wasm` - WebAssembly binary
- `moltis_wasm.d.ts` - TypeScript definitions
- `package.json` - npm package manifest

## Usage

### JavaScript/TypeScript

```javascript
import init, { MoltisClient, version } from 'moltis-wasm';

async function main() {
    // Initialize the WASM module
    await init();

    console.log(`moltis-wasm version: ${version()}`);

    // Create a client
    const client = new MoltisClient();

    // Configure backend (optional - for full functionality)
    client.setBackendUrl("http://localhost:3000");

    // Start a new session
    const sessionId = client.newSession();
    console.log(`Session: ${sessionId}`);

    // Parse protocol frames
    const frame = client.parseFrame('{"type":"message","content":"Hello"}');
    console.log(frame);
}

main();
```

### HTML

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { MoltisClient } from './pkg/moltis_wasm.js';

        async function run() {
            await init();
            const client = new MoltisClient();
            console.log('Client created:', client.getSessionId());
        }

        run();
    </script>
</head>
<body>
    <h1>Moltis WASM Demo</h1>
</body>
</html>
```

## Features

- `console-panic` - Enable better panic messages in the browser console (useful for debugging)

## What Works in WASM

- Protocol frame parsing and serialization
- Session management (in-memory)
- Configuration management
- UUID generation
- Logging (via tracing-wasm)

## What Requires Backend

These features are not available in pure WASM and require a backend gateway:

- Tool execution (shell commands, file operations)
- Sandbox management (Docker, containers)
- Browser automation (Chrome DevTools Protocol)
- File-based persistence (sessions, config, memory)
- Local embeddings (llama.cpp)
- Scheduled tasks (cron)

## Development

Run tests:
```bash
cargo test -p moltis-wasm
```

Run WASM tests in browser:
```bash
wasm-pack test --headless --chrome crates/wasm
```

## License

MIT
