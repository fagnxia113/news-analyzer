# LLM 分析配置说明

## 快速开始

### 1. 复制配置文件

```bash
copy .env.example .env
```

### 2. 编辑 .env 文件

打开 `.env` 文件，填入 API 配置：

```env
# 启用 LLM 分析
USE_LLM=true

# 使用 DeepSeek（推荐，性价比高）
LLM_PROVIDER=deepseek
LLM_API_KEY=你的API密钥
LLM_MODEL=deepseek-chat
```

### 3. 运行分析

```bash
npm run daily-llm
```

## 支持的 LLM 服务

### OpenAI（GPT-4o）
```env
USE_LLM=true
LLM_PROVIDER=openai
LLM_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxx
LLM_MODEL=gpt-4o-mini
```

### DeepSeek（性价比高，推荐）
```env
USE_LLM=true
LLM_PROVIDER=deepseek
LLM_API_KEY=sk-deepseek-xxxxxxxxxxxxxxxx
LLM_MODEL=deepseek-chat
```

### Anthropic Claude
```env
USE_LLM=true
LLM_PROVIDER=anthropic
LLM_API_KEY=sk-ant-xxxxxxxxxxxxxxxxxxxx
LLM_MODEL=claude-3-5-haiku-20241022
LLM_BASE_URL=https://api.anthropic.com
```

### 自定义服务
```env
USE_LLM=true
LLM_PROVIDER=openai
LLM_API_KEY=your-api-key
LLM_MODEL=your-model-name
LLM_BASE_URL=https://your-api-endpoint.com/v1
```

## 配置参数

| 参数 | 必填 | 说明 | 示例值 |
|------|------|------|--------|
| USE_LLM | 是 | 是否启用 LLM 分析 | true/false |
| LLM_PROVIDER | 否 | 服务提供商 | openai/deepseek/anthropic |
| LLM_API_KEY | 是 | API 密钥 | sk-xxxxxxx |
| LLM_MODEL | 否 | 模型名称 | gpt-4o-mini |
| LLM_BASE_URL | 否 | API 基础地址（会自动设置） | https://api.openai.com/v1 |

## 切换分析方式

### 使用 LLM 分析
- 设置 `USE_LLM=true`
- 填写有效的 `LLM_API_KEY`

### 使用规则分析
以下任一方式：
- 设置 `USE_LLM=false`
- 不填写 `LLM_API_KEY`
- 删除 `.env` 文件

## 常见问题

**Q: API 调用失败怎么办？**

A: 检查以下几项：
1. 确认 API 密钥正确
2. 确认 LLM_PROVIDER 设置正确
3. 检查网络连接
4. 确认模型名称正确

**Q: 如何降低成本？**

A: 使用更便宜的模型：
- **DeepSeek**: deepseek-chat（¥1/百万tokens，推荐）
- **OpenAI**: gpt-4o-mini（$0.15/百万tokens）

**Q: 配置文件存放在哪里？**

A: 配置文件存放在：`d:\业余兴趣\wechat-article-skill\.env`
