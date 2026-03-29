# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-03-29

### Added

- JavaScript scripting engine powered by QuickJS (rquickjs v0.11, ~1.5MB embedded)
- Pre-request scripts: run JS before sending to set env vars, compute auth tokens, etc.
- Test scripts: run JS after response to validate with `rl.test()` / `rl.expect()` assertions
- `rl` global API: `rl.environment.get/set`, `rl.request`, `rl.response`, `rl.response.json()`
- `rl.expect()` assertion chain: `toBe`, `toEqual`, `toContain`, `toBeDefined`, `toBeUndefined`, `toBeGreaterThan`, `toBeLessThan`, `toBeTruthy`
- `crypto` global: `crypto.sha256()`, `crypto.md5()`, `crypto.hmacSHA256()`
- `console.log()` with multi-type argument support (numbers, booleans, strings)
- Script tab in request editor with dual-pane editor (pre-request + test)
- Built-in dynamic variables: `$timestamp`, `$isoTimestamp`, `$guid`, `$randomInt`, `$randomEmail`, `$randomString`
- Unresolved variable detection (`find_unmatched`) excluding dynamic variables
- 5-second script execution timeout to prevent infinite loops
- Collection runner: full scripting integration with env propagation across requests
- Pre-request script errors abort the HTTP request instead of silently proceeding
- Tauri IPC command `execute_script` for frontend–backend script execution
- CI pre-verification workflow documented in CLAUDE.md (using `pnpm` scripts to match CI exactly)

## [0.6.0] - 2026-03-29

### Added

- Assertion system: declarative response validation rules (status code, response time, headers, JSONPath, body contains)
- Assertion editor UI with source/operator/expected grid in the new "Assert" tab
- Collection runner evaluates assertions to determine pass/fail (replaces default 2xx check when assertions exist)
- Runner results display per-assertion pass/fail details with actual vs expected values
- Runner results include response body preview (first 2KB) for failed requests
- Right-click context menu on requests with "Duplicate" and "Delete" actions
- CLI headless runner (`reqlight-cli`) for CI/CD integration with `--file`, `--collection`, `--env` flags
- JUnit XML report export via `--junit` flag in CLI
- Data-driven testing: CSV and JSON data file support via `--data` flag for parameterized test runs
- Rust-side assertion evaluator mirroring frontend logic for CLI test execution
- Unresolved variable warnings in editor response view, collection runner results, and CLI output
- Response header "Copy All" button and per-header copy-value buttons
- Toast confirmation feedback for response body and header copy actions
- `--fail-fast` flag in CLI to stop on first failure

### Fixed

- Assertion filter logic: incomplete assertions (missing expected value or source value) are no longer silently saved
- Context menu delete on requests now correctly deletes the request instead of looking up as collection
- Clippy `new_without_default` warning on `WsManager`

## [0.5.2] - 2026-03-29

### Changed

- Split `persistence.rs` tests into `persistence_tests.rs` (file was over 300-line limit)
- Split `types.ts` helper functions into `type-helpers.ts` (file was over 300-line limit)
- Disable syntax highlighting for response bodies exceeding 512 KB to prevent UI freeze
- Release workflow now auto-extracts release notes from CHANGELOG.md
- Require Node.js >=22 (`engines` + `.node-version`) to fix `vp check`/`vp fmt`/`vp lint` config loading
- Pre-commit hook uses `npx` instead of bare `vp` to resolve PATH issues

### Fixed

- `vp check`/`vp fmt`/`vp lint` failing locally due to Node.js 20 unable to load `.ts` config files

## [0.5.1] - 2026-03-29

### Added

- OAuth2 PKCE (S256) support for Authorization Code flow
- OAuth2 state parameter validation for CSRF protection
- Structured logging with daily rotating log files (tracing + tracing-appender)
- JSONPath wildcard (`[*]`) and recursive descent (`..`) support for response extraction
- Per-request timeout configuration (persisted with saved requests)
- Comprehensive test coverage: OAuth error paths, WebSocket edge cases, command layer tests

### Changed

- Structured error types via `thiserror` (`AppError` enum replaces raw `String` errors in services)
- HTTP client refactored from monolithic function into focused pipeline helpers
- Frontend auth/extraction logic deduplicated between editor and runner stores
- Magic constants centralized into dedicated `constants` module (Rust + TypeScript)

### Fixed

- Cargo.toml version out of sync with package.json and tauri.conf.json

## [0.5.0] - 2026-03-29

### Changed

- Migrate frontend toolchain from Vite 6 + Biome to Vite+ (Vite 8 + Oxlint + Oxfmt)
- Add pre-commit hooks via Vite+ staged config (replaces manual checks)
- Update CI workflow to use `vp` CLI instead of standalone pnpm/node setup

## [0.4.0] - 2026-03-29

### Added

- Proxy settings: configurable HTTP proxy with URL and no-proxy list
- Code generation: export requests as JavaScript fetch, axios, Python requests, or cURL
- Response variable extraction: define JSONPath rules to extract values into environment variables
- Collection runner: execute all requests in a collection sequentially with pass/fail tracking
- OAuth 2.0 authentication: Client Credentials and Authorization Code grant flows with token refresh
- OpenAPI 3.x spec import: parse JSON/YAML specs into request collections
- HAR file import: import HTTP Archive files as request collections
- Response snapshot diff: pin a response and compare side-by-side with the latest
- GraphQL request support: dedicated query and variables editor
- HEAD and OPTIONS HTTP methods
- WebSocket custom headers: attach headers to the WebSocket handshake
- WebSocket auto-reconnect: exponential backoff with max 5 attempts
- WebSocket environment variable interpolation in URLs and headers

### Changed

- CLAUDE.md unified to English with added commit discipline rules

## [0.3.3] - 2026-03-27

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
