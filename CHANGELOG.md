# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- WebSocket support (connect, send text messages, view message stream)
- Automatic Cookie jar (cookies persist across requests within a session)
- Error boundary with crash recovery UI
- Component tests for URLBar, ResponseView, CollectionItem
- Performance benchmark tests for stores and JSON highlighter
- Tauri auto-updater via GitHub Releases

### Fixed
- API Key with query location was not appended to the URL

## [0.1.0] - 2025-03-27

### Added
- HTTP requests: GET, POST, PUT, PATCH, DELETE
- Query parameters, custom headers, multiple body types (JSON, Form Data, Raw Text)
- Authentication: Bearer Token, Basic Auth, API Key (header or query)
- Collections: organize requests into folders, rename, duplicate, drag-to-reorder
- Environment variables: multiple environments, `{{variable}}` interpolation
- Secure storage: sensitive values stored in OS keychain (macOS Keychain / Windows Credential Manager / Linux Secret Service)
- cURL import & export with proper shell escaping
- JSON syntax highlighting with VS Code color scheme
- Request history (last 100 entries with full replay)
- Dark / Light theme (follows system preference or manual toggle)
- Keyboard shortcuts: `⌘↩` Send, `⌘N` New Request, `⌘⇧N` New Collection, `⌘E` Environments
- Response body truncation at 5MB with warning indicator
- Request cancellation via `tokio::select!` with `Arc<Notify>` signal
- Cross-platform CI (macOS Intel/ARM, Windows, Linux)
- Release workflow for cross-platform builds via GitHub Actions
