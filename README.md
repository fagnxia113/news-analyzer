# 微信公众号 AI 日报

AI Agent 驱动的微信公众号新闻分析工具。从订阅的公众号文章中提取新闻，AI 智能分类、去重、筛选，生成行业日报，自动排版后保存为微信公众号草稿，由用户自行审核发布。

基于 [wechat-article-exporter](https://github.com/nichenke/wechat-article-exporter) 提供微信文章数据接口。

## 完整工作流

```
收集文章 → AI 分析去重 → 生成 Markdown 日报 → 转微信 HTML → 剪贴板粘贴 → 微信公众号保存草稿 → 用户审核发布
```

1. **启动服务** — 运行 wechat-article-exporter，扫码登录
2. **收集文章** — 调用 API 获取指定时间段的公众号文章
3. **AI 分析** — Agent 读取文章 JSON，先提出候选核心观点，再逐条网络搜索验证，去重（对比最近7天日报）、分类、生成 Markdown 日报
4. **生成 HTML** — markdown-to-wechat.js 将 Markdown 转为微信兼容的内联样式 HTML
5. **粘贴排版** — 通过跨页面剪贴板方式将 HTML 粘贴到微信编辑器
6. **保存草稿** — 在微信公众号后台设置标题、替换正文、保存为草稿
7. **用户发布** — 用户审核草稿后手动发布

## 前置依赖

- **Node.js** — 运行 wechat-article-exporter 和分析脚本
- **AI Agent** — Trae、Claude Code、Codex 等，需支持 Chrome DevTools MCP
- **Chrome DevTools MCP** — 操作浏览器（排版和保存草稿阶段）
- **微信公众号** — 管理员权限，用于扫码登录和保存草稿

## 使用方式

对 Agent 说 **"分析新闻"** 或 **"发日报"**，自动完成从生成到保存草稿的全部操作。

**触发方式**：

| 说法 | 说明 |
|------|------|
| "分析新闻" 或 "发日报" | 昨天7:00到今天7:00（默认） |
| "分析从昨天下午到今天的新闻" | 自定义时间范围 |
| "分析5月30号的新闻" | 指定日期 |

**不同 Agent 的使用方式**：

| Agent | 说明 |
|-------|------|
| Trae | 直接说即可，自动读取 `AGENTS.md` |
| Claude Code | 直接说即可，自动读取 `CLAUDE.md` → `AGENTS.md` |
| Codex | 直接说即可，自动读取 `AGENTS.md` |
| 其他 | 首次说"请阅读 AGENTS.md，然后分析新闻"，后续直接说即可 |

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
- `AUTH_KEY` — 微信导出器的认证密钥（启动服务后访问 `/api/public/v1/authkey` 获取）
- `INDUSTRY_NAME` / `INDUSTRY_KEYWORDS` / `INDUSTRY_CATEGORIES` — 行业配置

### 3. 启动服务并登录

```bash
cd wechat-article-exporter
npx nuxt dev --port 3200
```

浏览器访问 http://localhost:3200 ，扫码登录微信公众号。

### 4. 运行

对 Agent 说 **"分析新闻"**

## 行业配置

通过 `.env` 文件适配不同行业，无需改代码：

| 变量 | 说明 | 示例 |
|------|------|------|
| `INDUSTRY_NAME` | 行业名称（用于标题和文件名，不含 emoji） | `算力，数据中心，AI` |
| `INDUSTRY_KEYWORDS` | 筛选关键词（逗号分隔） | `算力,数据中心,AI,GPU,芯片` |
| `INDUSTRY_CATEGORIES` | 分类体系（格式: 名称:描述） | `融资与投资:融资、投资等` |

> 文章标题采用双轨标题：`{当天最有关注度的事件+影响/悬念/冲突}｜M.D数据中心、算力、AI日报`，不含 emoji。标题和正文关键观点必须由 Agent 逐条网络搜索验证，避免使用过时信息或正文无法支撑的夸张表达。`INDUSTRY_CATEGORIES` 中的 emoji 仅用于正文分类标题装饰。

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
├── AGENTS.md                        # 工作流定义（所有 Agent 读取）
├── CLAUDE.md                        # Claude Code 入口（引用 AGENTS.md）
├── .trae/skills/wechat-editor/      # 微信编辑器操作 Skill
│   └── SKILL.md                     # ProseMirror 编辑器操作指南
├── wechat-article-exporter/         # 微信文章导出器（Nuxt.js，端口 3200）
├── wechat-article-skill/            # 分析脚本
│   ├── collect-articles.js          # 文章收集脚本
│   ├── daily-analysis-llm.js        # 日报分析主脚本（外部 LLM 模式）
│   ├── markdown-to-wechat.js        # Markdown → 微信兼容 HTML 转换
│   ├── weekly-analysis.js           # 周报分析脚本
│   ├── llm-analyzer.js              # LLM 分析核心模块
│   ├── auth-manager.js              # 认证管理
│   ├── config-manager.js            # 配置管理
│   ├── recent-articles.js           # 文章获取
│   └── .env.example                 # 配置模板
└── output/                          # 日报输出目录
```

## 日报格式

日报采用"判断优先、快慢结合"模式：

1. **今日判断** — 首屏先给一句当天核心判断，再列3个变化信号
2. **🔥 今日重点**（最多3条）— 慢热点做深，含详细摘要、"为什么重要"、"这意味着"、"受益方"、"承压方"、"后续观察"
3. **⚡ 一分钟速览** — 快资讯做极简，按分类用表格呈现（动态概要 | 影响度 | 原文链接）
4. **关注引导语** — 文末留存卡片，引导关注

### 增长优化原则

- Agent 生成的标题、今日判断、产业影响和受益/承压判断必须逐条网络搜索验证，优先引用官方公告、监管/交易所文件、公司新闻稿和权威媒体。
- 每个核心观点至少有1个可靠来源支撑；融资金额、政策、订单、投产、芯片/AI模型、公司战略等高时效内容优先用2个来源交叉确认。
- 无法验证或来源冲突的观点不得写入标题和今日判断；只能找到二手消息时，必须降级语气或删除。
- 标题前半句写成"事件 + 影响/悬念/冲突"，不要只堆公司名、金额或行业词。
- 弱新闻宁可删除，不为凑数量保留会议稿、软文稿和重复转述。
- 文章摘要建议写成当天最重要的一句话判断，封面围绕当天主事件，不使用纯装饰图。

### 去重规则

生成日报时自动与最近7天的日报去重，避免同一条新闻连续多天出现。同一事件有重大进展时保留并标注"（更新）"。

## 致谢

- [wechat-article-exporter](https://github.com/nichenke/wechat-article-exporter) — 提供微信公众号文章数据接口

## 许可证

MIT
