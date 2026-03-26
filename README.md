# Reqlight (Tauri)

A lightweight, cross-platform HTTP client — the [Tauri](https://tauri.app/) port of [Reqlight](https://github.com/puiks/reqlight) (native macOS version).

> **Reqlight** is a fast, minimal alternative to Postman. No login required. Fully offline. Your data stays local.

## Features

- **HTTP Requests** — GET / POST / PUT / PATCH / DELETE with query params, headers, and multiple body types (JSON, Form Data, Raw Text)
- **Collections** — Organize requests into folders, rename, duplicate, drag to reorder
- **Environment Variables** — Multiple environments (dev / staging / prod), `{{variable}}` interpolation everywhere
- **Secure Storage** — Sensitive values stored in OS keychain (macOS Keychain / Windows Credential Manager / Linux Secret Service)
- **cURL Import & Export** — Paste a cURL command to import, or export any request as cURL
- **JSON Syntax Highlighting** — Auto-formatted, color-coded response viewer
- **Request History** — Last 100 requests with status and timing
- **Dark / Light Theme** — Follows system preference, or toggle manually
- **Keyboard Shortcuts** — `⌘↩` Send, `⌘N` New Request, `⌘⇧N` New Collection, `⌘E` Environments

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust + [Tauri v2](https://v2.tauri.app/) |
| HTTP | [reqwest](https://docs.rs/reqwest) + rustls |
| Secrets | [keyring](https://docs.rs/keyring) (cross-platform) |
| Frontend | [Svelte 5](https://svelte.dev/) + TypeScript |
| Styling | Pure CSS + CSS Variables (zero dependencies) |
| Build | Vite + Cargo |

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

## Project Structure

```
reqlight-tauri/
├── src-tauri/               # Rust backend
│   └── src/
│       ├── commands/        # Tauri IPC commands (HTTP, persistence, keychain, cURL)
│       ├── models/          # Data models (serde, JSON-compatible with native app)
│       └── services/        # Business logic (HTTP client, interpolator, cURL parser)
├── src/                     # Svelte frontend
│   ├── components/          # UI components (sidebar, editor, response, environment)
│   ├── lib/
│   │   ├── stores/          # Svelte 5 rune-based state ($state, $derived)
│   │   ├── utils/           # JSON highlighter, keyboard shortcuts
│   │   ├── types.ts         # TypeScript type definitions
│   │   └── commands.ts      # Type-safe Tauri invoke wrappers
│   └── app.css              # Global styles + CSS design tokens
```

## Data Storage

- **Requests & collections**: `<app_data_dir>/data.json` (human-readable, pretty-printed)
- **Secret variables**: OS credential store (never written to disk)
- **Theme preference**: `localStorage`

## Related

- [**Reqlight (macOS native)**](https://github.com/puiks/reqlight) — The original SwiftUI version for macOS. Zero dependencies, ~30MB memory, instant launch.

## License

MIT
