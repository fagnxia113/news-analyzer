---
name: "link-validator"
description: "审计微信公众号日报中的链接合法性，识别 AI 幻觉链接并提供修复建议。当保存草稿出现'不合法链接'错误时调用。"
---

# 链接合法性审计工具

当微信公众号保存草稿出现"请勿输入不合法的消息图文链接"错误时，使用此 skill 定位并修复问题链接。

## When to Invoke

- 保存草稿时出现"不合法链接"错误
- 需要验证日报中所有链接是否来自真实收集的文章
- 需要统计链接复用情况

## 核心概念

### AI 幻觉链接
AI 在生成日报时可能虚构不存在的 URL（如 `https://mp.weixin.qq.com/s/xxxxx`），这些链接在 collected-articles.json 中找不到对应原文，微信编辑器会拒绝保存。

### 合法链接复用
汇总文章（如"每日大事件"）被拆分为多条独立新闻时，多条新闻指向同一 URL 是正常的，不应被当作问题处理。

## 使用步骤

### Step 1: 运行审计脚本

```bash
cd wechat-article-skill
node _audit-links.cjs
```

脚本会自动读取：
- `output/` 目录下最新的 .md 日报文件
- `output/` 目录下最新的 collected-articles-*.json 文件

### Step 2: 查看审计结果

脚本输出包含：
1. **AI幻觉的链接**：不在 collected-articles.json 中的 URL，这些是问题链接
2. **报告链接数 vs 收集的文章数**：对比统计
3. **被复用>1次的链接**：正常现象，不需要处理

### Step 3: 修复 AI 幻觉链接

在 .md 文件中，把 AI 幻觉链接对应的行改成纯文字：

修改前：
```
| 新闻标题 | ⭐⭐⭐ | [链接](https://mp.weixin.qq.com/s/幻觉URL) |
```

修改后：
```
| 新闻标题 | ⭐⭐⭐ | 原文（未收录） |
```

### Step 4: 重新生成 HTML

```bash
node markdown-to-wechat.js "output/日报文件名.md"
```

注意：**不要使用 --whitelist 参数**，默认行为会保留所有链接。

### Step 5: 重新粘贴到微信编辑器

参考 wechat-editor skill 的流程重新粘贴并保存草稿。

## 重要规则

1. **只修复 AI 幻觉链接**，不要删除合法链接
2. **链接复用不是问题**，同一 URL 被复用 10-20 次是正常的
3. **不要用 --whitelist 参数**，它会过度过滤
4. **每次修复后重新审计**，确认没有遗漏

## 相关文件

- `_audit-links.cjs`：链接审计脚本（对比 .md 与 collected-articles.json）
- `_check-urls.cjs`：URL 可达性检查（用 curl 验证链接是否返回 200）
- `_extract-links.cjs`：从 .md 文件提取所有链接
- `_unique-urls.cjs`：统计唯一 URL 和复用次数
- `validate-links.cjs`：完整链接验证流程
