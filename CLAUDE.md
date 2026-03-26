# Reqlight Tauri - Development Guidelines

## Tech Stack

- **Frontend**: Svelte 5 (runes) + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **Package Manager**: pnpm
- **Icons**: lucide-svelte

## Architecture & Directory Structure

```
src/                          # Svelte 5 Frontend
├── main.ts                   # App entry
├── App.svelte                # Root component (global shortcuts, loading)
├── app.css                   # Design tokens & global styles
├── components/               # UI components (organized by domain)
│   ├── editor/               # Request editor (URL bar, tabs, body)
│   ├── response/             # Response viewer (body, headers, status)
│   ├── sidebar/              # Collection tree, search, history
│   ├── environment/          # Environment variable management
│   ├── toolbar/              # Toolbar, cURL import
│   ├── layout/               # Layout shells
│   └── shared/               # Reusable primitives (Modal, Toast, etc.)
├── lib/
│   ├── commands.ts           # Tauri IPC wrappers (type-safe invoke)
│   ├── types.ts              # Shared TypeScript types
│   ├── constants.ts          # Magic numbers & config values
│   ├── stores/               # Svelte 5 rune-based state ($state, $derived)
│   │   ├── app.svelte.ts     # Collections, environments, history
│   │   └── editor.svelte.ts  # Active request editor state
│   └── utils/                # Pure utility functions
│       ├── json-highlighter.ts
│       └── keyboard.ts

src-tauri/src/                # Rust Backend
├── main.rs                   # Entry (delegates to lib.rs)
├── lib.rs                    # Tauri app setup + handler registration
├── commands/                 # Tauri IPC command handlers (thin layer)
│   ├── http.rs               # send_request
│   ├── persistence.rs        # load_state / save_state
│   ├── keychain.rs           # secret_get / secret_set / secret_delete
│   └── curl.rs               # parse_curl / export_curl
├── models/                   # Data structures (Serialize/Deserialize)
│   ├── request.rs            # SavedRequest, HttpMethod, RequestBody, KeyValuePair
│   ├── response.rs           # ResponseRecord
│   ├── collection.rs         # RequestCollection
│   ├── environment.rs        # RequestEnvironment
│   ├── history.rs            # RequestHistoryEntry
│   └── state.rs              # AppState (root persistence)
└── services/                 # Business logic (testable, no Tauri deps)
    ├── http_client.rs        # reqwest execution
    ├── persistence.rs        # File I/O + keychain integration
    ├── keychain.rs           # OS credential store wrapper
    ├── interpolator.rs       # {{variable}} replacement
    ├── curl_parser.rs        # cURL string → SavedRequest
    └── curl_exporter.rs      # SavedRequest → cURL string
```

## Development Rules

### TDD & Testing

- **所有新功能必须先写测试，再写实现（TDD）。**
- Rust 测试放在各模块文件底部的 `#[cfg(test)] mod tests {}` 中。
- 前端测试使用 vitest（如已配置），测试文件与源文件同目录，命名 `*.test.ts`。
- 提交前必须确保 `cargo test` 和 `pnpm check` 全部通过。
- 最低覆盖范围：所有 `services/` 模块必须有单元测试。

### File Organization

- **单文件不超过 300 行。** 超过时必须拆分。
- **按职责拆分，不按类型拆分。** 例如：把 cURL 解析和导出分成两个文件，而不是塞进一个 "curl_utils" 里。
- **组件文件遵循单一职责原则。** 一个 `.svelte` 文件只做一件事。
- **复用优先。** 提取共享逻辑到 `lib/utils/`（前端）或 `services/`（Rust）。
- 新目录需要有明确的领域边界，不要随意创建。

### Code Style

- Rust: 使用 `cargo fmt` 和 `cargo clippy`，零警告策略。
- Frontend: 遵循项目已有的 Svelte 5 runes 风格（`$state`, `$derived`, `$effect`）。
- 类型：TypeScript 使用 strict mode，Rust 中避免 `unwrap()`（除测试代码外）。

### Tauri IPC Convention

- 前端 → Rust 的调用统一走 `src/lib/commands.ts`，不直接调用 `invoke()`。
- 命令名 Rust 侧 snake_case，前端侧 camelCase，在 commands.ts 中映射。
- 新增 IPC 命令时，同步更新 `lib.rs` 的 handler 注册和 `commands.ts` 的类型包装。

### Data Flow

- 持久化数据存储在 `~/.../Reqlight/data.json`，秘密值存 OS keychain。
- 环境变量通过 `{{variable}}` 语法插值，处理在 Rust 侧完成。
- 前端 state 变更通过 debounced save（500ms）自动持久化。

## Build & Check Commands

```bash
pnpm dev              # Start dev (Vite + Tauri)
pnpm build            # Production frontend build
pnpm check            # svelte-check + TypeScript
pnpm tauri dev        # Full Tauri dev mode
pnpm tauri build      # Production Tauri build

# Rust (run from src-tauri/)
cargo check           # Type check
cargo test            # Run tests
cargo clippy          # Lint
cargo fmt             # Format
```
