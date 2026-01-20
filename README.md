# nrimp

**nrimp** is a high-performance Node.js HTTP client binding for the Rust `rquest` library. It enables **browser impersonation** (mimicking Chrome, Safari, Firefox, Edge TLS/JA3/JA4 fingerprints) to bypass sophisticated anti-bot systems, all while providing a simple TypeScript API.

> This package is a Node.js adaptation and fork of [primp](https://github.com/deedy5/primp), bringing browser impersonation capabilities to the Node.js ecosystem.

## Features

-   **Browser Impersonation**: Mimic real browser TLS fingerprints (JA3/JA4) and HTTP/2 behavior.
-   **High Performance**: Built on Rust (`rquest`, `tokio`, `boring`) and N-API.
-   **TypeScript Support**: Full type definitions included.
-   **Async/Await**: Non-blocking asynchronous API.
-   **HTTP/2 Support**: Built-in HTTP/2 support.

## Prerequisites

To build this package from source, you need:

1.  **Node.js**: v14+
2.  **Rust**: Stable toolchain (`rustup`)
3.  **CMake**: Required for building BoringSSL.
4.  **NASM**: Netwide Assembler (required for BoringSSL optimization).
5.  **LLVM/Clang**: Required for `bindgen`.

### Windows Setup (Example)
```bash
winget install LLVM.LLVM
winget install NASM.NASM
winget install Kitware.CMake
```

## Installation

```bash
npm install nrimp
```

*(Note: Currently requires building from source as prebuilds are not yet published)*

## Usage

```typescript
import { Client, HttpMethod } from 'nrimp';

async function main() {
    // Initialize client with Chrome 120 impersonation on Windows
    const client = new Client({
        impersonate: "chrome_120",
        impersonate_os: "windows",
        timeout: 30, // seconds
        verify: true // Verify SSL certificates
    });

    try {
        console.log("Sending request...");

        const response = await client.request(
            HttpMethod.GET,
            "https://tls.peet.ws/api/all",
            undefined, // params
            { "User-Agent": "Custom-Agent-If-Needed" } // headers
        );

        console.log("Status Code:", response.statusCode);
        console.log("Headers:", response.headers());

        // Parse JSON body
        const body = await response.json();
        console.log("JA3 Fingerprint:", body.tls.ja3);

    } catch (error) {
        console.error("Request failed:", error);
    }
}

main();
```

## API Reference

See [DOCS.md](DOCS.md) for detailed API documentation.

## License

MIT
