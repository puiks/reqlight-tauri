# Reqlight

A lightweight, cross-platform HTTP client built with [Tauri](https://tauri.app/).

> **Reqlight** is a fast, minimal alternative to Postman. No login required. Fully offline. Your data stays local.

## Features

- **HTTP Requests** — GET / POST / PUT / PATCH / DELETE / HEAD / OPTIONS with query params, headers, and multiple body types (JSON, Form Data, Multipart, Raw Text)
- **GraphQL** — Dedicated query and variables editor with syntax support
- **WebSocket** — Connect with custom headers, auto-reconnect with exponential backoff, environment variable interpolation
- **Authentication** — Bearer Token, Basic Auth, API Key, OAuth 2.0 (Client Credentials & Authorization Code flows)
- **Collections** — Organize requests into folders, rename, duplicate, drag to reorder
- **Collection Runner** — Execute all requests in a collection sequentially with pass/fail tracking and variable chaining
- **Environment Variables** — Multiple environments (dev / staging / prod), `{{variable}}` interpolation everywhere
- **Response Variable Extraction** — Define JSONPath rules to extract values from responses into environment variables
- **Code Generation** — Export requests as JavaScript (fetch / axios), Python (requests), or cURL
- **Import / Export** — cURL, Postman collections & environments, OpenAPI 3.x specs, HAR files
- **Response Diff** — Pin a response and compare side-by-side with the latest result
- **Proxy Settings** — Configurable HTTP proxy with URL and no-proxy list
- **Secure Storage** — Sensitive values stored in OS keychain (macOS Keychain / Windows Credential Manager)
- **Request History** — Last 100 requests with status, timing, and link back to source request
- **Dark / Light Theme** — Follows system preference, or toggle manually
- **Keyboard Shortcuts** — `⌘↩` Send, `⌘N` New Request, `⌘⇧N` New Collection, `⌘E` Environments

## Tech Stack

| Layer     | Technology                                             |
| --------- | ------------------------------------------------------ |
| Backend   | Rust + [Tauri v2](https://v2.tauri.app/)               |
| HTTP      | [reqwest](https://docs.rs/reqwest) + rustls            |
| WebSocket | [tokio-tungstenite](https://docs.rs/tokio-tungstenite) |
| Secrets   | [keyring](https://docs.rs/keyring) (cross-platform)    |
| Frontend  | [Svelte 5](https://svelte.dev/) + TypeScript           |
| Styling   | Pure CSS + CSS Variables (zero dependencies)           |
| Build     | Vite + Cargo                                           |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) (stable)
- Platform-specific Tauri dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
pnpm install
pnpm tauri dev
```

### Build for Production

```bash
pnpm tauri build
```

The output binary will be in `src-tauri/target/release/`.

## Data Storage

- **Requests & collections**: `<app_data_dir>/data.json` (human-readable, pretty-printed)
- **Secret variables**: OS credential store (never written to disk)
- **Theme preference**: `localStorage`

## License

MIT
