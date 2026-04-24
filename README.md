# NexaCode

跨平台终端 Code Agent，基于 Rust + Ratatui 构建。

## 功能特性

- 🤖 智能代码助手
- 🔧 MCP 协议支持
- ⚡ 内置 Skills 系统
- 📦 跨平台支持 (macOS / Linux / Windows)

## 项目状态

**Phase 1: 项目脚手架与基础 TUI**

- ✅ 1.1 项目初始化
- ⏳ 1.2 基础 TUI 框架
- ⏳ 1.3 基础视图组件

详细进度请查看 [FEATURES.md](./FEATURES.md)。

## 快速开始

### 前置要求

- Rust 1.70+
- Cargo

### 运行

```bash
cargo run
```

调试模式：

```bash
RUST_LOG=debug cargo run
```

## 项目架构

```
src/
├── main.rs          # 入口
├── app.rs           # 应用状态
├── tui/             # TUI 层
├── core/            # 核心逻辑
├── skills/          # Skills 系统
├── mcp/             # MCP 协议
├── infra/           # 基础设施
└── state/           # 状态管理
```

详细架构文档请查看 [ARCHITECTURE.md](./ARCHITECTURE.md)。

## License

MIT
