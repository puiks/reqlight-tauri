# Reqlight Roadmap

## v0.1 (Current) — Foundation

Core HTTP client with collections, environments, cURL import/export.

**Status**: Stable. 36 Rust tests, CI on 3 platforms, store architecture refactored.

---

## v0.2 — Usability & Interop

### Auth Support
- **Bearer Token**: 独立 Auth tab，自动注入 `Authorization` header
- **Basic Auth**: 用户名/密码输入，自动 Base64 编码
- **API Key**: 支持 header 或 query param 注入
- **OAuth 2.0**: Authorization Code / Client Credentials flow（需要内嵌 webview 授权页面）

### Postman Collection Import/Export
- 解析 Postman Collection v2.1 JSON 格式
- 映射 Postman 的 folder → Reqlight collection, request → request
- 环境变量映射（Postman `{{var}}` 语法已兼容）
- 导出为 Postman Collection JSON 供协作

### Tab-Style Multi-Request Editor
- 多标签页编辑器，支持同时打开多个请求
- 标签页状态独立（每个 tab 有自己的 editor state）
- 拖拽标签排序、关闭、固定
- 脏标记提示（未保存的变更）
- **架构影响**: 需要从单例 `editorStore` 改为 `Map<tabId, EditorState>` 模式

### Response Improvements
- 图片/PDF/二进制响应的预览
- Response body 搜索（Ctrl+F）
- 多响应对比（diff view）

---

## v0.3 — Advanced Protocol & Testing

### WebSocket Support
- 建立/断开 WebSocket 连接
- 发送/接收文本消息
- 二进制消息（ArrayBuffer）查看：hex dump + UTF-8 尝试解码
- 消息时间线视图（sent/received 区分）
- 连接状态指示器
- **技术方案**: Rust 侧用 `tokio-tungstenite`，通过 Tauri event 向前端推送消息

### UI 自动化测试
- Playwright 或 Tauri 内置 WebDriver 测试
- 覆盖核心 flow: 新建集合 → 新建请求 → 编辑 → 发送 → 查看响应
- cURL 导入 flow
- 环境变量切换 flow
- 暗黑模式切换

### SSE (Server-Sent Events) Support
- 类似 WebSocket 的实时消息流
- 自动重连
- Event type 过滤

---

## Backlog (No timeline)

- GraphQL 专用编辑器（schema introspection, 变量面板）
- gRPC 支持（需要 protobuf 编译）
- Pre-request / Post-response 脚本（JavaScript sandbox）
- Mock server（本地拦截和模拟响应）
- 团队协作（共享集合，需要后端服务）
- 插件系统
