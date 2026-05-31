# WeChat Article Skill

微信公众号文章分析脚本集，配合 [wechat-article-exporter](https://github.com/nichenke/wechat-article-exporter) 使用。

## 脚本说明

| 脚本 | 说明 |
|------|------|
| `daily-analysis-llm.js` | 日报分析（LLM 智能分析，推荐） |
| `weekly-analysis.js` | 周报分析 |
| `recent-articles.js` | 获取最近文章 |

## 配置

复制模板并编辑：

```bash
cp .env.example .env
```

### LLM 配置

| 变量 | 必填 | 说明 |
|------|------|------|
| `USE_LLM` | 是 | 启用 LLM 分析（`true`/`false`） |
| `LLM_PROVIDER` | 否 | 服务提供商（`openai`/`deepseek`/`anthropic`） |
| `LLM_API_KEY` | 是 | API 密钥 |
| `LLM_MODEL` | 否 | 模型名称 |
| `LLM_BASE_URL` | 否 | API 地址（自动设置） |

### 认证配置

| 变量 | 说明 |
|------|------|
| `AUTH_KEY` | 微信导出器认证密钥，4天过期，从 `http://localhost:3200/api/public/v1/authkey` 获取 |

### 行业配置

| 变量 | 说明 | 示例 |
|------|------|------|
| `INDUSTRY_NAME` | 行业名称 | `算力，数据中心，AI` |
| `INDUSTRY_KEYWORDS` | 筛选关键词 | `算力,数据中心,AI,GPU,芯片` |
| `INDUSTRY_CATEGORIES` | 分类体系 | `💰 融资与投资:融资、投资等,📊 其他:其他` |

## 使用

```bash
# 日报分析（默认：昨天7点到今天7点）
node daily-analysis-llm.js

# 自定义时间范围
node daily-analysis-llm.js "2026-05-29 07:00" "2026-05-30 07:00"
```

## 认证管理

Auth Key 有效期 4 天，过期后 API 返回 `invalid session`。

更新方式：编辑 `.env` 文件中的 `AUTH_KEY` 值。

获取新 Key：
1. 浏览器访问 http://localhost:3200 扫码登录
2. 访问 http://localhost:3200/api/public/v1/authkey
3. 将返回的 key 更新到 `.env`

## 注意事项

- 运行分析前确保 wechat-article-exporter 服务已启动
- 分析期间不要停止 exporter 服务
- `.session.json` 文件为自动生成，已在 `.gitignore` 中排除
