# 登录认证管理系统

## 概述

为了解决微信文章导出器登录密钥（约4天）过期的问题，新增了自动认证管理模块。

## 文件说明

### [auth-manager.js](d:\业余兴趣\wechat-article-skill\auth-manager.js)
认证管理核心模块，提供以下功能：

- **自动检测过期**: 检查登录密钥是否超过 4 天有效期
- **会话缓存**: 将登录密钥保存到 `.session.json` 文件
- **错误处理**: 自动检测 401/403 认证错误
- **重新登录提示**: 认证失败时给出清晰的重新登录指引
- **自动重试**: 检测到认证失败后重试请求

### 更新的脚本
- [weekly-analysis.js](d:\业余兴趣\wechat-article-skill\weekly-analysis.js) - 使用认证管理器
- [recent-articles.js](d:\业余兴趣\wechat-article-skill\recent-articles.js) - 使用认证管理器

## 工作原理

```
┌─────────────────────────────────────────────────────────────────┐
│                      API 请求流程                                 │
└─────────────────────────────────────────────────────────────────┘

发起请求
    │
    ▼
检查会话文件 (.session.json)
    │
    ├─ 存在且未过期 ──→ 使用缓存的密钥
    │
    └─ 不存在或过期 ──→ 使用默认密钥
                        │
                        ▼
                  发送 API 请求
                        │
          ┌─────────────┴─────────────┐
          │                           │
    请求成功 (200)            请求失败 (401/403)
          │                           │
          ▼                           ▼
    返回数据                清除会话文件
                            提示重新登录
                            重试一次请求
                            │
                            ▼
                      返回数据 或 抛出错误
```

## 使用方法

### 1. 首次使用

直接运行脚本，系统会使用默认密钥：

```bash
cd d:\业余兴趣\wechat-article-skill
node weekly-analysis.js
```

### 2. 检查会话状态

```bash
node auth-manager.js
```

输出示例：
```
=== 登录会话状态检查 ===

✓ 会话有效
  创建时间: 2026/1/19 10:30:00
  距现在: 10.5 小时
  距过期: 85.5 小时

测试 API 连接...
✓ API 连接成功，共 3 个公众号
```

### 3. 认证失败时的处理

当检测到 401 或 403 错误时：

```
============================================================
❌ 认证失败！需要重新获取登录密钥
============================================================

请按以下步骤操作：
1. 打开微信文章导出器客户端
2. 重新登录或刷新页面
3. 在客户端中找到 Auth Key / 令牌
4. 将新的 AUTH_KEY 复制以下位置：
   - 本脚本中的 DEFAULT_AUTH_KEY 变量
   - 或手动输入新的密钥

提示：某些客户端支持持续登录，可能不需要此操作
============================================================
```

### 4. 手动清除会话

如果需要强制重新认证：

```bash
node -e "require('./auth-manager.js').clearSession()"
```

或直接删除 `.session.json` 文件。

## 配置说明

### auth-manager.js 中的配置

```javascript
// 默认的登录密钥（如果不需要登录）
const DEFAULT_AUTH_KEY = '04a6edcefc764689840478fbb76b6f13';

// 登录密钥的有效期（天）
const AUTH_KEY_EXPIRY_DAYS = 4;
```

### 修改默认密钥

如果微信文章导出器返回了新的 Auth Key，编辑 [auth-manager.js](d:\业余兴趣\wechat-article-skill\auth-manager.js) 中的 `DEFAULT_AUTH_KEY` 变量。

### 修改有效期

根据实际情况调整 `AUTH_KEY_EXPIRY_DAYS`。

## 自动化重新登录（高级）

如果你的微信文章导出器支持通过代码获取新的 Auth Key，可以扩展 `auth-manager.js`：

```javascript
/**
 * 自动获取新的 Auth Key
 */
async function fetchNewAuthKey() {
  try {
    // 这里实现自动获取密钥的逻辑
    // 例如：调用 API、解析页面、调用浏览器自动化等
    const newAuthKey = await yourCustomLoginFunction();

    if (newAuthKey) {
      await saveSession(newAuthKey);
      return newAuthKey;
    }
  } catch (error) {
    console.error('自动获取密钥失败:', error.message);
  }

  return null;
}

// 在 relogin 函数中使用
async function relogin() {
  const authKey = await fetchNewAuthKey();

  if (authKey) {
    console.log('✓ 自动重新登录成功');
    return true;
  }

  // 如果失败，显示手动登录提示
  // ... (原有代码)
}
```

## 文件结构

```
wechat-article-skill/
├── auth-manager.js        # 认证管理模块
├── .session.json          # 会话缓存文件（自动生成）
├── weekly-analysis.js     # 周报分析（已集成认证管理）
├── recent-articles.js     # 最近文章查询（已集成认证管理）
└── README-AUTH.md         # 本说明文档
```

## 注意事项

1. **`.session.json` 不要手动提交到版本控制** - 建议在 `.gitignore` 中添加：
   ```
   .session.json
   ```

2. **安全性** - `DEFAULT_AUTH_KEY` 是明文存储，在生产环境中应考虑：
   - 使用环境变量
   - 使用加密存储
   - 使用密钥管理服务

3. **多环境支持** - 如果有多个微信文章导出器实例，可以：
   - 使用不同的 `SESSION_FILE` 路径
   - 使用不同的 `BASE_URL`
   - 使用环境变量切换配置

## 故障排查

### 问题：API 连接失败 (ECONNREFUSED)

**原因**: 微信文章导出器未运行或端口错误

**解决**:
1. 检查微信文章导出器是否正在运行
2. 确认端口是否为 3000
3. 检查防火墙设置

### 问题：认证失败 (401/403) 且无法自动恢复

**原因**: 密钥已过期且自动刷新机制不可用

**解决**:
1. 手动打开微信文章导出器重新登录
2. 获取新的 Auth Key 并更新 `DEFAULT_AUTH_KEY`
3. 运行 `node auth-manager.js` 测试连接

### 问题：会话一直过期

**原因**: `AUTH_KEY_EXPIRY_DAYS` 设置过小

**解决**: 修改 [auth-manager.js](d:\业余兴趣\wechat-article-skill\auth-manager.js) 中的有效期设置。

## 向后兼容性

原有的脚本仍然可以独立运行（使用硬编码的 `AUTH_KEY`），只需要确保密钥有效即可。

使用 `auth-manager.js` 是可选的，但强烈推荐使用以获得更好的用户体验。
