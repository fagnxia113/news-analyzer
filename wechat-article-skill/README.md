# WeChat Article Exporter + News Analysis Suite

这是一个用于在 Claude Code 中控制 [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 系统，并进行自动化新闻分析的工具集。

## 功能特性

### 基础功能
- **登录/刷新**: 获取登录二维码并检查登录状态
- **公众号管理**: 搜索和管理微信公众号
- **文章列表**: 获取指定公众号的文章列表，支持关键词搜索和数量控制
- **文章详情**: 展示文章标题、作者、发布日期、链接、版权状态等信息
- **评论获取**: 获取文章的评论和回复

### 新闻分析功能（使用 LLM 智能分析）

- 使用大语言模型进行新闻分析
- 更准确的行业和类型分类
- 更好的摘要生成
- 支持多种 LLM 服务（OpenAI、Claude、DeepSeek 等）
- 自动拆分包含多条新闻的文章
- 按行业和新闻类型自动分类
- 生成日报和周报

*详细配置请参考 [LLM-SETUP.md](LLM-SETUP.md)*

## 前置要求

1. 运行 [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 服务器
   - 默认地址: `http://localhost:3000`
   - 或者使用在线服务: `https://down.mptext.top`

2. 安装 Node.js (>=18.0.0)

## 安装

```bash
cd wechat-article-skill
npm install
```

## 使用方法

### 新闻分析脚本

```bash
# 日报分析（使用 LLM 智能分析）
node daily-analysis-llm.js

# 或使用 npm script
npm run daily-llm
```

**详细配置 LLM 分析请查看 [LLM-SETUP.md](LLM-SETUP.md)**
**需要先配置 `.env` 文件中的 LLM 相关设置**

**日报分析功能**：
- 自动获取昨天18点至今天18点的文章
- LLM 智能拆分包含多条新闻的文章
- 更准确的行业分类（数据中心、算力、云计算、人工智能、大数据、跨境数据）
- 更准确的新闻类型分类（融资投资、政策法规、市场动态、技术创新、财务报告、战略合作、会展信息、项目动态）
- 生成 Markdown 格式的存档报告
- 智能过滤非新闻内容

### 通过命令行直接使用（基础功能）

```bash
# 查看帮助
npm start help

# 获取登录二维码
npm start login

# 检查登录状态
npm start check-login

# 搜索公众号
npm start search "微信公众号名称"

# 获取文章列表
npm start articles MzI...fakeid 5

# 带搜索关键词获取文章
npm start articles MzI...fakeid "文章关键词" 10

# 获取账号信息
npm start account-info

# 退出登录
npm start logout

# 获取文章评论
npm start comment 123456789
```

### 环境变量配置

```bash
# 设置 wechat-article-exporter 服务器地址
export WECHAT_EXPORTER_URL=http://localhost:3000

# 或使用在线服务
export WECHAT_EXPORTER_URL=https://down.mptext.top
```

## 命令说明

### 新闻分析命令

| 命令 | 说明 | 分析方式 |
|------|------|----------|
| `daily-analysis-llm.js` | 分析昨日18点至今日18点的新闻 | LLM 智能分析 |

### LLM 智能分析配置

1. 复制配置文件：
```bash
copy .env.example .env
```

2. 编辑 `.env` 文件：
```env
# 启用 LLM 分析
USE_LLM=true

# 使用 DeepSeek（推荐，性价比高）
LLM_PROVIDER=deepseek
LLM_API_KEY=你的API密钥
LLM_MODEL=deepseek-chat
```

3. 运行分析：
```bash
npm run daily-llm
```

**详细配置请查看 [LLM-SETUP.md](LLM-SETUP.md)**

支持的 LLM 服务：
- **OpenAI**: GPT-4o, GPT-4o-mini
- **DeepSeek**: deepseek-chat（性价比高）
- **Anthropic**: Claude 3.5 Haiku
- **其他**: 任何兼容 OpenAI API 格式的服务

### 基础功能命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `login` / `get-qrcode` | 获取登录二维码 | `npm start login` |
| `check-login` | 检查登录状态 | `npm start check-login` |
| `logout` | 退出登录 | `npm start logout` |
| `search <keyword>` | 搜索公众号 | `npm start search "技术公众号"` |
| `articles <fakeid> [keyword] [size]` | 获取文章列表 | `npm start articles MzI...fakeid` |
| `account-info` | 获取当前登录账号信息 | `npm start account-info` |
| `comment <commentId>` | 获取文章评论 | `npm start comment 123456789` |
| `help` | 显示帮助信息 | `npm start help` |

## 使用示例

### 新闻分析完整流程

```bash
# 1. 确保 wechat-article-exporter 服务正在运行
cd wechat-article-exporter
npm run dev

# 2. 配置 LLM（首次使用）
cd ../wechat-article-skill
# 编辑 .env 文件，设置 USE_LLM=true 和 LLM_API_KEY

# 3. 运行每日新闻分析
node daily-analysis-llm.js

# 4. 查看生成的报告
# 报告保存在：d:\业余兴趣\新闻分析结果\
# - Markdown 格式：wechat-daily-{timestamp}.md
```

### 基础功能完整流程

```bash
# 1. 获取登录二维码
npm start login
# 扫描二维码登录

# 2. 检查登录状态
npm start check-login

# 3. 搜索公众号
npm start search "阮一峰的网络日志"

# 4. 获取文章列表（使用返回的 fakeid）
npm start articles MjM5MDE...

# 5. 搜索特定文章
npm start articles MjM5MDE... "前端" 10

# 6. 退出登录
npm start logout
```

## 项目结构

```
wechat-article-skill/
├── auth-manager.js       # 认证管理模块
├── config-manager.js     # 配置管理模块
├── daily-analysis-llm.js # 日报分析脚本（LLM智能分析）
├── weekly-analysis.js    # 周报分析脚本
├── llm-analyzer.js       # LLM分析模块
├── recent-articles.js    # 获取最近文章
├── .env                  # 环境变量配置（需自行创建）
├── .env.example          # 环境变量模板
├── LLM-SETUP.md          # LLM配置说明
├── package.json          # 项目配置
└── README.md             # 项目文档
```

## 输出报告示例

新闻分析会生成 Markdown 格式的报告：

### Markdown 格式（用于存档）
- 结构化的文本格式
- 包含今日要闻分析
- 便于版本控制
- 支持各种编辑器查看

## 自定义配置

### LLM 服务提供商

编辑 `.env` 文件，可以选择不同的 LLM 服务提供商：

支持的 LLM 服务：
- **OpenAI**: GPT-4o, GPT-4o-mini
- **DeepSeek**: deepseek-chat（性价比高）
- **Anthropic**: Claude 3.5 Haiku
- **其他**: 任何兼容 OpenAI API 格式的服务（如 yovole、阿里云等）

## 注意事项

1. 首次使用需要先扫码登录微信公众号平台
2. 登录 token 有有效期，可能需要定期刷新
3. 运行新闻分析前，先启动 wechat-article-exporter 服务器
4. 新闻分析结果会自动保存到指定目录
5. 请遵守微信平台的使用规则
6. 使用 LLM 分析需要配置 `.env` 文件中的 API 密钥

## 许可证

MIT

## 相关链接

- [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter)
- [在线服务](https://down.mptext.top)
- [文档站点](https://docs.mptext.top)
