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

## Known Pitfalls / 已知踩坑点

> 这些是开发过程中踩过的真实 bug，列在这里避免再犯。

### Shell 字符串转义
- **cURL 导出时必须转义单引号。** 用 `'it'\''s'` 语法（先关单引号、加转义单引号、再开单引号）。
- 绝对不要直接把用户输入内嵌到 `format!("'{}'", user_input)` 里——单引号会破坏 shell 语法。

### Debounced Save 与窗口关闭
- `scheduleSave()` 用 500ms debounce，关窗口时 pending 的 save 可能还没触发。
- **必须在 `beforeunload` 里调用 `flushSave()`**，强制立即写入。
- `editorStore.saveIfDirty()` 只保存编辑器状态，`appStore.flushSave()` 才保存集合/环境/历史。两个都要调。

### HTTP Header 大小写
- `reqwest::HeaderMap` 是大小写不敏感的——`contains_key("content-type")` 和 `contains_key("Content-Type")` 结果相同。
- **但必须用 `reqwest::header::CONTENT_TYPE` 常量做 key lookup**，不要用字符串字面量，避免歧义和 clippy 警告。

### cURL 解析器的 Method 推断
- `-d` 数据 flag 应自动将 GET 升级为 POST，**但前提是没有 `-X` 显式指定 method 且没有 `-G` flag**。
- `-G` flag 强制 GET，但不应覆盖 `-X` 显式指定的 method。
- 要用三个 flag（`explicit_method`、`has_data`、`force_get`）协同判断，不能在解析循环内直接改 method。

### 响应体大小
- **必须限制响应体读取大小**（当前上限 5MB）。不限的话，大响应会卡死前端 JSON 渲染。
- 超限时截断并设置 `is_truncated` 标记，前端应展示警告。

### Tauri 异步命令取消
- Tauri v2 的 `#[tauri::command]` 不支持原生取消。要实现取消需要：
  1. 在 `lib.rs` 用 `.manage()` 注册一个 `Arc<Notify>` 信号
  2. 在 `send_request` 里用 `tokio::select!` 竞争执行和取消信号
  3. 前端调用单独的 `cancel_request` 命令来触发信号
- **不要用假取消**（只设 `isLoading = false`）——后端请求还在跑，资源没释放。

### Rust enum Default derive
- 如果 enum 的 Default impl 只是返回第一个 variant，**用 `#[derive(Default)]` + `#[default]` 属性**，不要手写 `impl Default`。
- Clippy `derivable_impls` lint 会报错（在 `-D warnings` 模式下是 hard error）。

### pub use 重导出
- `mod.rs` 里的 `pub use submodule::*` 如果没有外部代码通过它引用，会产生 unused import 警告。
- 只重导出实际需要从模块外访问的类型，不要无脑 `pub use *`。

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
