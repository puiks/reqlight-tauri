# Reqlight - Development Guidelines

## Tech Stack

- **Frontend**: Svelte 5 (runes) + TypeScript + Vite+ (Vite 8 + Vitest + Oxlint + Oxfmt)
- **Backend**: Rust + Tauri v2
- **Toolchain**: Vite+ (`vp` CLI) — unified dev/test/lint/fmt
- **Package Manager**: pnpm (managed by `vp`)
- **Icons**: CSS-based (inline SVG)

## Architecture & Directory Structure

```
src/                          # Svelte 5 Frontend
├── main.ts                   # App entry
├── App.svelte                # Root component (global shortcuts, modal state)
├── app.css                   # Design tokens & global styles
├── components/               # UI components (organized by domain)
│   ├── editor/               # Request editor (URL bar, tabs, body, auth, extraction)
│   ├── response/             # Response viewer (body, headers, status)
│   ├── sidebar/              # Collection tree, search, history
│   ├── environment/          # Environment variable management
│   ├── toolbar/              # Toolbar, cURL import
│   ├── layout/               # Layout shells (MainLayout)
│   ├── codegen/              # Code generation modal
│   ├── runner/               # Collection runner modal
│   ├── settings/             # App settings (proxy config)
│   └── shared/               # Reusable primitives (Modal, Toast, etc.)
├── lib/
│   ├── commands.ts           # Tauri IPC wrappers (type-safe invoke)
│   ├── types.ts              # Shared TypeScript types + helpers
│   ├── constants.ts          # Magic numbers & config values
│   ├── stores/               # Svelte 5 rune-based state ($state, $derived)
│   │   ├── observable.svelte.ts # Base class: observer pattern for save scheduling
│   │   ├── app.svelte.ts     # Collections, environments, history, proxy config
│   │   ├── editor.svelte.ts  # Active request editor state + extraction
│   │   ├── environment.svelte.ts # Environment store (setVariable for extraction)
│   │   ├── runner.svelte.ts  # Collection runner orchestration
│   │   ├── history.svelte.ts # History management
│   │   ├── toast.svelte.ts   # Toast notifications
│   │   └── websocket.svelte.ts # WebSocket connection state
│   └── utils/                # Pure utility functions
│       ├── html.ts           # Shared escapeHtml (single source of truth)
│       ├── json-highlighter.ts
│       ├── xml-highlighter.ts
│       ├── jsonpath.ts       # Simple JSONPath extraction (dot notation + array index)
│       ├── text-search.ts    # Fuzzy text search
│       ├── errors.ts         # Error handling utilities
│       └── keyboard.ts       # Keyboard shortcut registration

src-tauri/src/                # Rust Backend
├── main.rs                   # Entry (delegates to lib.rs)
├── lib.rs                    # Tauri app setup + handler registration
├── commands/                 # Tauri IPC command handlers (thin layer)
│   ├── http.rs               # send_request (with proxy support)
│   ├── persistence.rs        # load_state / save_state
│   ├── keychain.rs           # secret_get / secret_set / secret_delete
│   ├── curl.rs               # parse_curl / export_curl
│   ├── codegen.rs            # generate_code
│   ├── collection_io.rs      # import_collection / export_collection
│   └── websocket.rs          # ws_connect / ws_send / ws_disconnect
├── models/                   # Data structures (Serialize/Deserialize)
│   ├── request.rs            # SavedRequest, HttpMethod, RequestBody, KeyValuePair
│   ├── response.rs           # ResponseRecord
│   ├── collection.rs         # RequestCollection
│   ├── environment.rs        # RequestEnvironment
│   ├── history.rs            # RequestHistoryEntry
│   ├── auth.rs               # AuthConfig (none/bearer/basic/apiKey)
│   ├── extraction.rs         # ExtractionRule (JSONPath variable extraction)
│   ├── proxy.rs              # ProxyConfig
│   └── state.rs              # AppState (root persistence)
├── test_utils.rs             # Shared test helpers (make_kv, etc.) — #[cfg(test)] only
└── services/                 # Business logic (testable, no Tauri deps)
    ├── http_client.rs        # reqwest execution (tests in http_client_tests.rs)
    ├── persistence.rs        # File I/O + keychain integration
    ├── keychain.rs           # OS credential store wrapper
    ├── interpolator.rs       # {{variable}} replacement
    ├── curl_parser.rs        # cURL string → SavedRequest
    ├── curl_exporter.rs      # SavedRequest → cURL string
    ├── code_generator.rs     # Multi-language code snippet generation
    ├── collection_import.rs  # Postman collection import
    ├── collection_export.rs  # Postman collection export
    ├── collection_io.rs      # File dialog helpers for import/export
    ├── collection_types.rs   # Shared Postman data structures
    └── websocket.rs          # WebSocket connection manager
```

## Development Rules

### TDD & Testing

- **All new features must have tests written before implementation (TDD).**
- Rust tests go in `#[cfg(test)] mod tests {}` at the bottom of each module file. If tests push the file beyond 300 lines, extract them to a `_tests.rs` file.
- Frontend tests use vitest + jsdom, co-located with source files, named `*.test.ts`.
- Before committing, ensure `cargo test`, `vp test run`, and `vp check` all pass.
- **Rust minimum coverage**: All modules under `services/` that contain pure logic must have unit tests. Pure I/O wrappers (e.g., keychain) are exempt but must include a comment explaining why.
- **Frontend minimum coverage**: All `lib/utils/` modules must have unit tests. **Every new `.ts` utility file must have a corresponding `.test.ts` — no exceptions.**
- Frontend component tests (optional) use `@testing-library/svelte`, focusing on components with complex interaction logic.
- **Do not duplicate Rust test helpers across files.** Shared helpers go in `test_utils.rs`, imported via `use crate::test_utils::*`.

### File Organization

- **No single file exceeds 300 lines.** Split when exceeded. Rust test lines count toward the total — if tests push the file over 300 lines, extract them to a `_tests.rs` file (using `#[cfg(test)] #[path = "xxx_tests.rs"] mod tests;`).
- **Split by responsibility, not by type.** For example: separate cURL parsing and exporting into two files, rather than combining them into one "curl_utils" file.
- **Component files follow the single responsibility principle.** One `.svelte` file does one thing.
- **Prefer reuse.** Extract shared logic to `lib/utils/` (frontend) or `services/` (Rust).
- New directories must have clear domain boundaries — do not create them arbitrarily.

### Code Reuse & Deduplication

- **The same utility function must not be implemented in multiple places.** If two or more files need the same functionality (e.g., `escapeHtml`, `make_kv`), extract it to a shared module.
  - Frontend shared utilities → `lib/utils/` (e.g., `html.ts`)
  - Rust shared test helpers → `test_utils.rs` (`#[cfg(test)]` gated)
  - Store shared logic → `stores/observable.svelte.ts` (base class inheritance)
- **Before adding a new utility function, search for existing implementations.** Use `grep` to search for function name keywords to avoid accidentally reinventing the wheel.

### Dependency Hygiene

- **Do not introduce unused dependencies.** Every new dependency must have a corresponding `use`/`import`.
- **Clean up regularly:** Remove crates (Cargo.toml) and npm packages (package.json) that are no longer referenced in code.
- Before committing changes to dependency files, verify that all dependencies are actively used.

### Code Style

- Rust: Use `cargo fmt` and `cargo clippy` with a zero-warning policy.
- Frontend: Use `vp fmt` (Oxfmt) for formatting and `vp lint` (Oxlint) for linting. Follow the project's existing Svelte 5 runes style (`$state`, `$derived`, `$effect`).
- Types: TypeScript uses strict mode; Rust avoids `unwrap()` (except in test code).

### Commit Discipline

- **Atomic commits.** Each commit should represent one logical change (one feature, one bug fix, one refactor). Do not bundle unrelated changes into a single commit.
- **Commit messages must be in English.** Use the conventional format: a short imperative summary line (≤72 chars), optionally followed by a blank line and a longer description.
- **Use gitmoji prefix** for commit type: `✨` (feat), `🐛` (fix), `♻️` (refactor), `🧪` (test), `📝` (docs), `🔖` (release), `🎨` (style/format), `🔧` (config).
- **Commit at natural boundaries** — after completing a self-contained piece of work, not after accumulating a large batch. If a feature spans Rust + frontend, it is fine to commit them together as one atomic unit, but do not mix in unrelated changes.

### Pre-Commit Checklist

> **All checks must pass before every commit — no skipping steps. CI enforces these strictly.**

```bash
# Rust (run from src-tauri/)
cargo fmt --check          # Format check (CI will fail on diff)
cargo clippy -- -D warnings  # Zero warnings
cargo test                   # All passing

# Frontend (via Vite+)
vp lint                      # Oxlint — zero warnings
vp test run                  # All passing
vp check                     # svelte-check + TypeScript zero errors
```

- **`cargo fmt --check` is the most commonly forgotten.** Always run `cargo fmt` after modifying Rust code before committing.
- Performance test thresholds must account for CI environments (2-3x slower than local). For known-slow operations, use `{ timeout: 30000 }` with relaxed thresholds.

### Modal & Callback Threading Pattern

- **Modal state is managed centrally in `App.svelte`.** Each modal has a corresponding `showXxx = $state(false)` and `<XxxModal onclose={...} />`.
- **Event callbacks thread down from App:** `App.svelte` → `MainLayout` → `Sidebar`/`ResponseView` → child components. Each layer passes callbacks via `$props()`.
- **Do not directly import App-level modal state in child components.** Maintain unidirectional data flow: children notify parents via callbacks, parents control modal visibility.

### Tauri IPC Convention

- All frontend → Rust calls go through `src/lib/commands.ts` — never call `invoke()` directly.
- Command names use snake_case on the Rust side and camelCase on the frontend side, mapped in commands.ts.
- When adding a new IPC command, update both `lib.rs` handler registration and `commands.ts` type wrappers in sync.

### Data Flow

- Persistent data is stored in `~/.../Reqlight/data.json`; secrets are stored in the OS keychain.
- Environment variables use `{{variable}}` syntax for interpolation, processed on the Rust side.
- Frontend state changes are auto-persisted via debounced save (500ms).

## Known Pitfalls

> These are real bugs encountered during development — listed here to prevent recurrence.

### Shell String Escaping

- **cURL export must escape single quotes.** Use the `'it'\''s'` syntax (close quote, escaped quote, reopen quote).
- Never embed user input directly into `format!("'{}'", user_input)` — single quotes will break shell syntax.

### Debounced Save & Window Close

- `scheduleSave()` uses a 500ms debounce; pending saves may not fire when the window closes.
- **Must call `flushSave()` in `beforeunload`** to force an immediate write.
- `editorStore.saveIfDirty()` only saves editor state; `appStore.flushSave()` saves collections/environments/history. Both must be called.

### HTTP Header Case Sensitivity

- `reqwest::HeaderMap` is case-insensitive — `contains_key("content-type")` and `contains_key("Content-Type")` return the same result.
- **Always use `reqwest::header::CONTENT_TYPE` constants for key lookups** — avoid string literals to prevent ambiguity and clippy warnings.

### cURL Parser Method Inference

- The `-d` data flag should auto-upgrade GET to POST, **but only if `-X` has not explicitly set a method and `-G` flag is not present**.
- The `-G` flag forces GET but should not override an explicitly specified `-X` method.
- Use three flags (`explicit_method`, `has_data`, `force_get`) to determine the final method — do not modify the method directly inside the parsing loop.

### Response Body Size

- **Response body read size must be limited** (current cap: 5MB). Without a limit, large responses will freeze the frontend JSON renderer.
- When truncated, set the `is_truncated` flag; the frontend should display a warning.

### Tauri Async Command Cancellation

- Tauri v2's `#[tauri::command]` does not support native cancellation. To implement cancellation:
  1. Register an `Arc<Notify>` signal via `.manage()` in `lib.rs`
  2. Use `tokio::select!` in `send_request` to race execution against the cancel signal
  3. The frontend calls a separate `cancel_request` command to trigger the signal
- **Do not fake cancellation** (just setting `isLoading = false`) — the backend request continues running, leaking resources.

### Rust Enum Default Derive

- If an enum's Default impl simply returns the first variant, **use `#[derive(Default)]` + `#[default]` attribute** instead of a manual `impl Default`.
- The clippy `derivable_impls` lint will error (hard error under `-D warnings` mode).

### pub use Re-exports

- `pub use submodule::*` in `mod.rs` produces unused import warnings if no external code references it.
- Only re-export types that are actually accessed from outside the module — do not blindly `pub use *`.

### Serde Backward Compatibility (Persisted Model Extension)

- **When adding new fields to persisted structs, always add `#[serde(default)]`.** Otherwise, old data files will fail to deserialize, causing users to lose all data.
- Vec-typed fields are naturally compatible (`Vec::default()` is an empty array), but scalar types other than Option need explicit `#[serde(default)]` or `#[serde(default = "...")]`.
- After adding a field, search all manual construction sites for that struct (test helpers, import logic, etc.) and add initialization for the new field.

### Clippy Idiomatic Patterns

- **Use `is_some_and(|x| ...)` instead of `map_or(false, |x| ...)`.** The clippy `unnecessary_map_or` lint will error.
- **Use `is_none_or(|x| ...)` instead of `map_or(true, |x| ...)`.** Same reasoning.

## Build & Check Commands

```bash
# Frontend (via Vite+)
vp dev                # Start dev server
vp build              # Production frontend build
vp check              # svelte-check + TypeScript
vp test run           # Run frontend unit tests (vitest)
vp test               # Watch mode for frontend tests
vp test run --coverage  # Frontend tests with coverage report
vp lint               # Oxlint linting
vp fmt                # Oxfmt formatting
vp preview            # Preview production build
pnpm tauri dev        # Full Tauri dev mode
pnpm tauri build      # Production Tauri build

# Rust (run from src-tauri/)
cargo check           # Type check
cargo test            # Run tests
cargo clippy          # Lint
cargo fmt             # Format
```

## Release & Versioning

### Version Number Management

Three version numbers must stay in sync, handled automatically by `scripts/bump.sh`:

- `package.json` → `"version"`
- `src-tauri/Cargo.toml` → `version`
- `src-tauri/tauri.conf.json` → `"version"`

**Never manually edit version numbers in these three files.**

### CHANGELOG Convention

- File: `CHANGELOG.md`, following the [Keep a Changelog](https://keepachangelog.com/) format.
- During development, all user-visible changes should be recorded in the `[Unreleased]` section.
- Categories: `Added` (new features), `Changed` (behavior changes), `Fixed` (bug fixes), `Removed` (removals).
- `bump.sh` automatically renames `[Unreleased]` to the version number + date, and creates a new empty `[Unreleased]` section.

### Release Flow

```bash
# 1. Ensure CHANGELOG.md [Unreleased] is fully populated
# 2. Ensure all checks pass (pre-commit checklist)
# 3. Run version release (auto-updates version numbers + CHANGELOG + creates commit + tag)
./scripts/bump.sh 0.X.0

# 4. Push to remote to trigger CI/CD
git push --follow-tags
```

### CI/CD Pipeline

- **CI (`.github/workflows/ci.yml`)**: Runs automatically on PRs and pushes — fmt, clippy, cargo test, vp lint, vp test, vp check, Playwright E2E.
- **Release (`.github/workflows/release.yml`)**: Triggered automatically when pushing a `v*` tag — builds macOS (Intel + ARM) and Windows installers, creates a GitHub Draft Release.
- Draft Releases must be manually reviewed and published on GitHub.

### Version Number Convention

- Feature iterations use minor versions (0.4.0 → 0.5.0).
- Bug fixes use patch versions (0.5.0 → 0.5.1).
- Publish 1.0.0 upon reaching the feature-complete milestone.
