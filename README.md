# News Analyzer Skill

微信公众号新闻分析 **Skill**（技能）。从订阅的公众号文章中提取新闻，通过 LLM 智能分类和筛选，生成行业日报，排版后保存为微信公众号草稿，由用户自行审核发布。

> **什么是 Skill？** Skill 是一套指令集 + 脚本 + 配置，告诉 AI Agent 如何执行特定任务。Agent（如 Claude Code、Codex、Trae）读取 Skill 后按流程执行。本项目是 Skill，不是 Agent。

基于 [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 提供微信文章数据接口。

## 完整工作流

本 Skill 覆盖从分析到保存草稿的完整流程，最终由用户自行审核发布：

```
获取文章 → LLM 分析 → 生成日报 → 花生编辑器排版 → 微信公众号保存为草稿 → 用户审核发布
```

具体步骤：

1. **启动服务** — 运行 wechat-article-exporter，扫码登录
2. **生成日报** — Agent 调用 API 获取文章，LLM 分析生成 Markdown 日报
3. **排版** — 在花生编辑器（https://editor.huasheng.ai/）中粘贴内容，转为微信公众号格式
4. **保存草稿** — 在微信公众号后台替换标题和正文，保存为草稿
5. **用户发布** — 用户登录微信公众号后台，审核草稿后手动发布

## 前置依赖

### 必需

- **Node.js** — 运行 wechat-article-exporter 和分析脚本
- **AI Agent** — 执行本 Skill 的 Agent，支持 Claude Code、Codex、Gemini、Trae、Cline 等
- **Chrome DevTools MCP** — Agent 操作浏览器的工具（排版和保存草稿阶段必须，仅方式一需要）
- **微信公众号** — 需要管理员权限，用于扫码登录和保存草稿

### 可选

- **LLM API Key** — 仅"方式二"需要，方式一由 Agent 自身完成分析

## 两种使用方式

### 方式一：Agent 端到端执行（零配置，推荐）

对 Agent 说 **"分析新闻"** 或 **"发日报"**，Agent 读取本 Skill 后自动完成从生成到保存草稿的全部操作，最终由用户审核发布。

- 调用 exporter API 获取文章
- 利用自身 LLM 能力完成分析（无需额外 API Key）
- 通过 Chrome DevTools MCP 操作浏览器完成排版和保存草稿

**触发方式**：

| 说法 | 说明 |
|------|------|
| "分析新闻" 或 "发日报" | 分析昨天7:00到今天7:00的新闻（默认时间范围） |
| "分析从昨天下午到今天的新闻" | 自定义时间范围，Agent 会调整脚本参数 |
| "分析5月30号的新闻" | 指定日期，Agent 会计算对应时间范围 |

**不同 Agent 的使用方式**：

| Agent | 说明 |
|-------|------|
| Claude Code | 直接说即可，自动读取 `CLAUDE.md` → `AGENTS.md` |
| Codex | 直接说即可，自动读取 `AGENTS.md` |
| 其他 Agent | 首次使用时说"请阅读 AGENTS.md，然后分析新闻"，后续直接说即可 |

**所需配置**：
- 启动 wechat-article-exporter 服务
- 配置 Auth Key（首次扫码登录后写入 `.env`，有效期4天，过期后脚本会报错再重新获取）
- 配置行业参数（`INDUSTRY_NAME` / `INDUSTRY_KEYWORDS` / `INDUSTRY_CATEGORIES`）
- Chrome DevTools MCP 已连接

**不需要**：`LLM_API_KEY`、`LLM_PROVIDER`、`LLM_MODEL`

**适合**：日常使用，分析质量高，零额外成本，全自动保存草稿

### 方式二：脚本独立运行（自定义 API）

运行 `daily-analysis-llm.js`，脚本自行调用外部 LLM API 完成分析，仅生成 Markdown 日报文件。排版和保存草稿需手动操作或由 Agent 后续执行。

**所需配置**：
- 启动 wechat-article-exporter 服务
- 获取 Auth Key
- 配置行业参数
- **额外配置** `LLM_API_KEY`、`LLM_PROVIDER`、`LLM_MODEL`

**适合**：定时任务、批量处理、CI/CD 集成

## 快速开始

### 1. 安装依赖

```bash
cd wechat-article-exporter && npm install
cd ../wechat-article-skill && npm install
```

### 2. 配置

```bash
cd wechat-article-skill
cp .env.example .env
```

编辑 `.env`，填入：
- `AUTH_KEY` — 微信导出器的认证密钥（启动服务后获取）
- `INDUSTRY_NAME` / `INDUSTRY_KEYWORDS` / `INDUSTRY_CATEGORIES` — 行业配置
- `LLM_API_KEY` / `LLM_PROVIDER` / `LLM_MODEL` — 仅方式二需要

### 3. 启动服务并登录

```bash
cd wechat-article-exporter
npx nuxt dev --port 3200
```

浏览器访问 http://localhost:3200 ，扫码登录微信公众号。

### 4. 运行

**方式一**：对 Agent 说 **"分析新闻"**

Agent 读取本 Skill 后将自动执行：生成日报 → 花生编辑器排版 → 微信公众号保存为草稿 → 用户审核发布

**方式二**：
```bash
cd wechat-article-skill
node daily-analysis-llm.js
```

报告输出到 `output/` 目录。排版和保存草稿需手动操作。

## 行业配置

通过 `.env` 文件适配不同行业，无需改代码：

| 变量 | 说明 | 示例 |
|------|------|------|
| `INDUSTRY_NAME` | 行业名称（用于标题和文件名，不含 emoji） | `算力，数据中心，AI` |
| `INDUSTRY_KEYWORDS` | 筛选关键词（逗号分隔） | `算力,数据中心,AI,GPU,芯片` |
| `INDUSTRY_CATEGORIES` | 分类体系（格式: emoji 名称:描述） | `融资与投资:融资、投资等` |

> **注意**：`INDUSTRY_CATEGORIES` 中的 emoji 仅用于日报正文的分类标题装饰，不会出现在微信公众号的文章标题中。文章标题格式为 `{INDUSTRY_NAME}动态日报 YYYY年M月D号`，不含任何 emoji。

### 行业示例

**医疗健康**：
```env
INDUSTRY_NAME=医疗健康
INDUSTRY_KEYWORDS=医疗,健康,医药,生物,器械,临床
INDUSTRY_CATEGORIES=融资与投资:融资、投资等,医疗政策:政策法规等,新药研发:新药审批、临床试验等,风险:安全事件等,其他:其他
```

**新能源**：
```env
INDUSTRY_NAME=新能源
INDUSTRY_KEYWORDS=光伏,储能,新能源,电池,充电桩,碳中和
INDUSTRY_CATEGORIES=融资与投资:融资、投资等,光伏:光伏技术、项目等,储能:储能技术、项目等,风险:安全事件等,其他:其他
```

## 项目结构

```
├── AGENTS.md                    # Skill 工作流定义（通用，所有 Agent 读取）
├── CLAUDE.md                    # Claude Code 专用入口（引用 AGENTS.md）
├── wechat-article-exporter/     # 微信文章导出器（Nuxt.js，端口 3200）
├── wechat-article-skill/        # 分析脚本
│   ├── daily-analysis-llm.js    # 日报分析主脚本（方式二）
│   ├── weekly-analysis.js       # 周报分析脚本
│   ├── llm-analyzer.js          # LLM 分析核心模块
│   ├── auth-manager.js          # 认证管理
│   ├── config-manager.js        # 配置管理（LLM + 行业）
│   ├── recent-articles.js       # 文章获取
│   └── .env.example             # 配置模板
└── output/                      # 日报输出目录
```

## 支持的 LLM 服务（方式二）

支持任何 OpenAI 兼容 API，只需配置 `LLM_BASE_URL`、`LLM_API_KEY`、`LLM_MODEL`：

| 服务 | base_url | model |
|------|----------|-------|
| OpenAI | `https://api.openai.com/v1` | `gpt-4o-mini` |
| DeepSeek | `https://api.deepseek.com/v1` | `deepseek-chat` |
| Anthropic | 需使用 `anthropic` provider | `claude-3-5-haiku-20241022` |
| 其他 OpenAI 兼容 | 填入对应的 base_url | 填入对应的 model |

## 致谢

- [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) — 提供微信公众号文章数据接口，本项目的基础依赖

## 许可证

MIT
