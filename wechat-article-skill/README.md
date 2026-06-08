# WeChat Article Skill

微信公众号文章分析脚本集，配合 [wechat-article-exporter](https://github.com/nichenke/wechat-article-exporter) 使用。

## 脚本说明

| 脚本 | 说明 |
|------|------|
| `collect-articles.js` | 收集文章数据（不调用LLM，由AI Agent分析） |
| `daily-analysis-llm.js` | 日报分析（外部LLM API已禁用，保留代码供参考） |
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
# 新流程：收集文章（不调用LLM）
node collect-articles.js

# 然后 AI Agent 读取 output/collected-articles-MMDD.json，先提出候选核心观点，再逐条网络搜索验证后完成分析

# 旧流程（外部LLM API已禁用）
node daily-analysis-llm.js

# 自定义时间范围
node collect-articles.js "2026-05-29 07:00" "2026-05-30 07:00"
```

## 日报格式

日报采用"一个主线、少量速览、强相关读者"模式：

1. **双轨标题**：`{变化/冲突/后果+行业关键词}｜M.D数据中心、算力、AI日报`（不含 emoji，前半句优先不超过22个中文字符）
2. **今日判断**：首屏先给"今天最重要的一句话"，再列3个变化信号，必须像人话、可转发
3. **今天对谁有用**：3-4条短句点名读者角色，以及他们该看什么
4. **🔥 今日主线**（慢热点，做深）：默认只写1条主线，用当天最强事件统领2-3个证据，含"发生了什么"、"为什么重要"、"谁该关注"、"下一步看什么"
5. **⚡ 一分钟速览**（快资讯，做极简）：全篇最多6-8条，表格形式，列：动态概要 | 影响度 | 原文链接
6. **关注引导语**：文末留存卡片，引导关注

### 增长优化规则

- Agent 生成的标题、今日判断、今日主线、为什么重要、谁该关注、下一步看什么必须逐条网络搜索验证，优先使用官方公告、监管/交易所文件、公司新闻稿和权威媒体。
- 每个核心观点至少有1个可靠来源支撑；融资金额、政策、订单、投产、芯片/AI模型、公司战略等高时效内容优先用2个来源交叉确认。
- 无法验证或来源冲突的观点不得写入标题和今日判断；只能找到二手消息时，必须降级语气或删除。
- 标题要有打开理由，但不能标题党；优先写"变化/冲突/后果"，不要只堆公司名和金额。
- 今日主线要写成一个故事，不要把相关事件拆成三张平行投研卡片。
- 摘要保留事实链条，判断部分回答"行业变化是什么、谁该关注、下一步看什么"。
- 一分钟速览只保留有明确产业影响的新闻，弱新闻宁可删除，全篇最多6-8条。

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
