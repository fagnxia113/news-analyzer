# News Analyzer

微信公众号新闻自动分析工具。从订阅的公众号文章中提取新闻，通过 LLM 智能分类和筛选，生成行业日报，一键发布到微信公众号。

基于 [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 提供微信文章数据接口。

## 功能

- 自动获取微信公众号文章
- LLM 智能提取新闻、去重、分类
- AI 筛选今日重点新闻（含深度解读）
- 生成 Markdown 格式日报
- 花生编辑器排版 → 微信公众号一键发布
- 行业可配置，适配任何领域

## 两种使用方式

### 方式一：AI 助手直接分析（零配置，推荐）

通过 Claude Code 等 AI 助手使用，无需配置 LLM API Key。AI 助手直接调用 exporter API 获取文章，利用自身 LLM 能力完成分析。

**所需配置**：
- 启动 wechat-article-exporter 服务
- 获取 Auth Key（扫码登录后从 API 获取）
- 配置行业参数（`INDUSTRY_NAME` / `INDUSTRY_KEYWORDS` / `INDUSTRY_CATEGORIES`）

**不需要**：`LLM_API_KEY`、`LLM_PROVIDER`、`LLM_MODEL`

**适合**：日常使用，分析质量高，零额外成本

### 方式二：脚本独立运行（自定义 API）

运行 `daily-analysis-llm.js`，脚本自行调用外部 LLM API 完成分析，适合自动化和定时任务。

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

### 4. 运行分析

**方式一**：在 Claude Code 中说"分析新闻"

**方式二**：
```bash
cd wechat-article-skill
node daily-analysis-llm.js
```

报告输出到 `output/` 目录。

## 行业配置

通过 `.env` 文件适配不同行业，无需改代码：

| 变量 | 说明 | 示例 |
|------|------|------|
| `INDUSTRY_NAME` | 行业名称（用于标题和文件名） | `算力，数据中心，AI` |
| `INDUSTRY_KEYWORDS` | 筛选关键词（逗号分隔） | `算力,数据中心,AI,GPU,芯片` |
| `INDUSTRY_CATEGORIES` | 分类体系（格式: emoji 名称:描述） | `💰 融资与投资:融资、投资等` |

### 行业示例

**医疗健康**：
```env
INDUSTRY_NAME=医疗健康
INDUSTRY_KEYWORDS=医疗,健康,医药,生物,器械,临床
INDUSTRY_CATEGORIES=💰 融资与投资:融资、投资等,🏥 医疗政策:政策法规等,💊 新药研发:新药审批、临床试验等,⚠️ 风险:安全事件等,📊 其他:其他
```

**新能源**：
```env
INDUSTRY_NAME=新能源
INDUSTRY_KEYWORDS=光伏,储能,新能源,电池,充电桩,碳中和
INDUSTRY_CATEGORIES=💰 融资与投资:融资、投资等,☀️ 光伏:光伏技术、项目等,🔋 储能:储能技术、项目等,⚠️ 风险:安全事件等,📊 其他:其他
```

## 项目结构

```
├── wechat-article-exporter/    # 微信文章导出器（Nuxt.js，端口 3200）
├── wechat-article-skill/       # 分析脚本
│   ├── daily-analysis-llm.js   # 日报分析主脚本（方式二）
│   ├── weekly-analysis.js      # 周报分析脚本
│   ├── llm-analyzer.js         # LLM 分析核心模块
│   ├── auth-manager.js         # 认证管理
│   ├── config-manager.js       # 配置管理（LLM + 行业）
│   ├── recent-articles.js      # 文章获取
│   └── .env.example            # 配置模板
├── output/                     # 日报输出目录
└── CLAUDE.md                   # AI 助手工作流配置
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
