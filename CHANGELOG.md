# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-03-27

### Added
- Contextual action placement: cURL import in URL bar, collection I/O in sidebar header
- Hover delete buttons (✕) on collections and requests with confirmation dialog
- Search now matches collection names (shows all child requests when matched)
- Show/hide eye toggle on auth secret fields (token, password, API key)
- History entries linked to source requests — clicking navigates back to the original request
- History shows request name, relative time, and elapsed time
- Linked history entries visually marked with blue left border

### Changed
- Default editor:response panel ratio from 50:50 to 70:30
- Toolbar buttons enlarged with hover states; removed low-frequency options from URL bar
- Environment picker gear icon replaced with "Manage" text button
- Collection/history chevron icons enlarged for better readability

### Fixed
- History replay not rendering in editor (requestId was set to null)

## [0.3.1] - 2026-03-27

### Added
- Custom app icon
- WebSocket support (connect, send text messages, view message stream)
- Automatic Cookie jar (cookies persist across requests within a session)
- Error boundary with crash recovery UI
- Tauri auto-updater via GitHub Releases
- Postman collection and environment import/export
- Multipart/form-data file upload support
- Collection drag-drop reordering
- Multi-format response rendering and in-body search
- Redirect control toggle

### Changed
- Removed Ubuntu/Linux from CI and release builds
- Removed low-value tests
- Refactored large files, extracted shared utils, cleaned dependencies

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
