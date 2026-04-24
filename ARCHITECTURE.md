# NexaCode 架构设计

## 概述

NexaCode 是一个跨平台终端 Code Agent，灵感来自 Claude Code 和 OpenCode，提供智能代码编辑、任务规划和开发辅助能力。

## 系统架构图

```
┌─────────────────────────────────────────────────────────┐
│                   TUI Presentation Layer                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │Chat View │  │Task View │  │Files View│  │Terminal│ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Layout Manager (Ratatui)            │  │
│  └──────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                  Application Core Layer                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Task Engine  │  │  Planning    │  │  Skills      │  │
│  │ (Orchestrator)│  │  Engine      │  │  Manager     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│  ┌──────────────┐  ┌──────────────┐                     │
│  │   Agent      │  │  Command     │                     │
│  │ Controller   │  │  Registry    │                     │
│  └──────────────┘  └──────────────┘                     │
├─────────────────────────────────────────────────────────┤
│                    Skills Layer                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Skill Registry                       │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ Built-in │ │  Project │ │  User    │ │ Dynamic  │  │
│  │  Skills  │ │  Skills  │ │  Skills  │ │  Skills  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Skill Execution Engine               │  │
│  └──────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                MCP & Tool Execution Layer               │
│  ┌──────────────────────────────────────────────────┐  │
│  │          MCP Protocol Handler                    │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Tools   │ │ Resources│ │ Prompts  │ │  Samplers│  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
├─────────────────────────────────────────────────────────┤
│                Infrastructure Layer                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ LLM API  │ │  FS      │ │  Git     │ │  Shell   │  │
│  │ Client   │ │  Watcher │ │  Client  │ │  Exec    │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────┘
```

## 核心模块详解

### 1. TUI Presentation Layer（展示层）

使用 **Ratatui** + **Crossterm** 构建跨平台终端界面。

#### 主要视图组件

| 组件 | 职责 |
|------|------|
| `ChatView` | 对话界面，展示 Agent 回复和用户输入 |
| `TaskView` | 任务列表和执行状态 |
| `FilesView` | 文件树和打开的文件 |
| `TerminalView` | 嵌入式终端 |
| `LayoutManager` | 布局管理、焦点切换、键盘事件路由 |

### 2. Application Core Layer（应用核心层）

#### Task Engine (Orchestrator)
负责协调整个 Agent 的执行流程：
- 接收用户输入
- 调用 Planning Engine 生成执行计划
- 调度任务执行
- 管理执行上下文

#### Planning Engine
将用户需求分解为可执行的多步计划：
- 需求分析
- 任务分解
- 依赖关系管理
- 计划验证

#### Agent Controller
核心推理逻辑：
- LLM 交互管理
- 工具调用决策
- 上下文构建
- 流式响应处理

#### Command Registry
命令注册和分发：
- 内置命令（:quit, :help 等）
- 动态命令注册
- 命令别名

### 3. Skills Layer（技能层）

#### Skill 数据结构

```rust
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,

    // 触发条件
    pub triggers: Vec<Trigger>,

    // 技能定义
    pub definition: SkillDefinition,

    // 元数据
    pub tags: Vec<String>,
    pub icon: Option<char>,
}

pub enum SkillDefinition {
    // 提示词模板
    Prompt(PromptSkill),
    // 工具组合
    Composite(CompositeSkill),
    // 自定义逻辑
    Custom(CustomSkill),
    // 流水线
    Pipeline(PipelineSkill),
}

pub enum Trigger {
    // 命令触发 ("/skill-name")
    Command(String),
    // 语义匹配
    Semantic(SemanticTrigger),
    // 文件模式
    FilePattern(Glob),
    // 事件触发
    Event(EventTrigger),
}
```

#### 内置 Skills

| Skill | 触发 | 功能 |
|-------|------|------|
| `/commit` | Command | 生成智能提交信息 |
| `/review` | Command | 代码审查 |
| `/refactor` | Command | 重构建议与执行 |
| `/test` | Command | 生成/运行测试 |
| `/explain` | Command | 代码解释 |
| `/docs` | Command | 生成文档 |
| `/debug` | Semantic | 调试助手 |
| `/architect` | Semantic | 架构设计 |

#### Skill 目录结构

```
project/
├── .claude/
│   ├── skills/
│   │   ├── custom-skill-1/
│   │   │   ├── skill.json
│   │   │   ├── prompt.md
│   │   │   └── actions.rs
│   │   └── my-skill/
│   └── config.json
```

### 4. MCP & Tool Execution Layer（MCP 协议层）

兼容 **Model Context Protocol**：
- Tools: 可调用的工具
- Resources: 可读取的资源
- Prompts: 提示词模板
- Samplers: 自定义采样逻辑

#### 工具执行沙箱
- 安全隔离
- 执行超时控制
- 变更追踪与回滚
- 权限管理

### 5. Infrastructure Layer（基础设施层）

| 模块 | 职责 |
|------|------|
| `llm-client` | LLM API 客户端（支持多提供商） |
| `fs-watcher` | 文件系统监听 |
| `git-client` | Git 操作 |
| `shell-exec` | 安全的命令执行 |

## 应用状态管理

### 主状态结构

```rust
pub struct App {
    // UI 状态
    pub current_mode: Mode,
    pub active_tab: Tab,
    pub focus: FocusArea,

    // 聊天/对话状态
    pub messages: Vec<Message>,
    pub input_buffer: String,
    pub is_streaming: bool,

    // 会话与历史
    pub session: Session,
    pub conversation_history: Vec<Conversation>,

    // 规划与任务状态
    pub current_plan: Option<Plan>,
    pub task_queue: TaskQueue,
    pub execution_context: ExecutionContext,

    // Agent 状态
    pub agent_state: AgentState,
    pub tasks: Vec<Task>,
    pub tool_calls: Vec<ToolCall>,

    // 项目上下文
    pub project_index: ProjectIndex,
    pub context_budget: ContextBudget,
    pub current_path: PathBuf,
    pub file_tree: FileNode,
    pub open_files: Vec<FileBuffer>,

    // MCP 状态
    pub mcp_servers: Vec<McpServerState>,
    pub available_tools: Vec<Tool>,

    // Skills 状态
    pub skill_manager: SkillManager,
    pub active_skills: Vec<ActiveSkill>,
    pub skill_invocations: Vec<SkillInvocation>,

    // 终端状态
    pub terminal_output: Vec<Line>,
    pub terminal_mode: TerminalMode,

    // Undo/Redo
    pub history: UndoRedoStack,

    // 配置
    pub config: Config,
}

pub enum Mode {
    Normal,      // 浏览模式
    Input,       // 输入模式
    Command,     // 命令模式（: 前缀）
    Search,      // 搜索模式（/ 前缀）
}

pub enum AgentState {
    Idle,
    Thinking,
    ExecutingTool,
    StreamingResponse,
    Error,
}

pub struct SkillManager {
    pub registry: HashMap<String, Skill>,
    pub loaded_paths: Vec<PathBuf>,
    pub skill_config: SkillConfig,
}
```

## 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| TUI 框架 | Ratatui + Crossterm | 活跃维护，跨平台支持好 |
| 异步运行时 | Tokio | 生态成熟，性能优秀 |
| MCP 协议 | 完全兼容 | 便于复用现有工具生态 |
| 技能系统 | 四层结构（内置/项目/用户/动态） | 灵活可扩展 |
| 状态管理 | Reducer 模式 + 单向数据流 | 可预测，易调试 |

## 技术栈

| 类别 | 技术 |
|------|------|
| 语言 | Rust |
| TUI | Ratatui, Crossterm |
| 异步 | Tokio |
| 序列化 | Serde, JSON |
| 配置 | Figment |
| 日志 | Tracing |
| LLM | 通用客户端（支持多提供商） |

## 目录结构

```
src/
├── main.rs                 # 入口点
├── app.rs                  # 主应用状态
├── tui/                    # TUI 层
│   ├── mod.rs
│   ├── components/         # UI 组件
│   ├── views/              # 视图
│   ├── layout.rs           # 布局管理
│   └── event.rs            # 事件处理
├── core/                   # 核心层
│   ├── mod.rs
│   ├── agent.rs            # Agent 控制器
│   ├── task_engine.rs      # 任务引擎
│   ├── planning.rs         # 规划引擎
│   └── command.rs          # 命令系统
├── skills/                 # Skills 层
│   ├── mod.rs
│   ├── manager.rs          # Skill 管理器
│   ├── registry.rs         # Skill 注册
│   ├── executor.rs         # Skill 执行引擎
│   └── builtin/            # 内置 Skills
├── mcp/                    # MCP 协议层
│   ├── mod.rs
│   ├── protocol.rs         # MCP 协议
│   ├── tools.rs            # 工具管理
│   ├── resources.rs        # 资源管理
│   └── sandbox.rs          # 执行沙箱
├── infra/                  # 基础设施层
│   ├── mod.rs
│   ├── llm/                # LLM 客户端
│   ├── fs/                 # 文件系统
│   ├── git/                # Git 操作
│   └── shell/              # Shell 执行
└── state/                  # 状态管理
    ├── mod.rs
    ├── actions.rs          # Action 定义
    ├── reducers.rs         # Reducers
    └── history.rs          # Undo/Redo
```
