#!/usr/bin/env node

/**
 * 带登录管理的 API 请求工具
 * 支持自动检测过期并重新登录
 */

import fs from 'fs/promises';
import path from 'path';

const BASE_URL = 'http://127.0.0.1:3200';
// 默认的登录密钥 (2026-03-18 更新)
const DEFAULT_AUTH_KEY = 'de4e7917d2a14493a99c7ff108901846';

// 会话文件路径
const SESSION_FILE = path.join(new URL('.', import.meta.url).pathname, '.session.json');

// 登录密钥的有效期（天）
const AUTH_KEY_EXPIRY_DAYS = 4;

/**
 * 读取会话信息
 */
async function loadSession() {
  try {
    const content = await fs.readFile(SESSION_FILE, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    return null;
  }
}

/**
 * 保存会话信息
 */
async function saveSession(authKey) {
  const session = {
    authKey,
    createdAt: Date.now(),
  };
  await fs.writeFile(SESSION_FILE, JSON.stringify(session, null, 2));
  console.log('✓ 会话已保存');
}

/**
 * 清除会话信息
 */
async function clearSession() {
  try {
    await fs.unlink(SESSION_FILE);
    console.log('✓ 会话已清除');
  } catch (error) {
    // 文件不存在也视为成功
  }
}

/**
 * 检查会话是否过期
 */
function isSessionExpired(session) {
  if (!session) return true;

  const ageInMs = Date.now() - session.createdAt;
  const ageInDays = ageInMs / (1000 * 60 * 60 * 24);

  return ageInDays >= AUTH_KEY_EXPIRY_DAYS;
}

/**
 * 获取有效的 auth key
 */
async function getAuthKey() {
  const session = await loadSession();

  if (session && !isSessionExpired(session)) {
    console.log('✓ 使用缓存的登录密钥');
    return session.authKey;
  }

  if (session && isSessionExpired(session)) {
    console.log('⚠ 登录密钥已过期，需要重新登录');
  }

  // 尝试使用默认密钥
  console.log('⚠ 使用默认密钥 (可能已过期)');
  return DEFAULT_AUTH_KEY;
}

/**
 * 重新登录（提示用户手动操作）
 */
async function relogin() {
  console.log('\n' + '='.repeat(60));
  console.log('❌ 认证失败！需要重新获取登录密钥');
  console.log('='.repeat(60));
  console.log('\n请按以下步骤操作：');
  console.log('1. 打开微信文章导出器客户端');
  console.log('2. 重新登录或刷新页面');
  console.log('3. 在客户端中找到 Auth Key / 令牌');
  console.log('4. 将新的 AUTH_KEY 复制以下位置：');
  console.log('   - 本脚本中的 DEFAULT_AUTH_KEY 变量');
  console.log('   - 或手动输入新的密钥\n');
  console.log('提示：某些客户端支持持续登录，可能不需要此操作');
  console.log('='.repeat(60) + '\n');

  return false;
}

/**
 * 带错误处理和自动重试的请求函数
 */
async function request(url, options = {}, retryCount = 0) {
  const authKey = await getAuthKey();

  // 提取 body 对象用于判断是否需要 Content-Type
  const body = options.body;

  const response = await fetch(url, {
    ...options,
    headers: {
      ...(body && { 'Content-Type': 'application/json' }),
      'x-auth-key': authKey,
      ...options.headers,
    },
  });

  // 检查响应状态
  if (!response.ok) {
    const errorMessage = `Request failed: ${response.status} ${response.statusText}`;
    console.error(`✗ ${errorMessage}`);

    // 检查是否是认证错误（401 或 403）
    if ((response.status === 401 || response.status === 403) && retryCount === 0) {
      console.log('⚠  检测到认证失败，尝试清除会话...');

      await clearSession();

      // 如果用户需要重新登录
      const success = await relogin();
      if (!success) {
        throw new Error('认证失败，请重新登录');
      }

      // 递归重试一次
      console.log('🔄 重试请求...');
      return request(url, options, retryCount + 1);
    }

    throw new Error(errorMessage);
  }

  const data = await response.json();

  // 检查业务状态码
  if (data.base_resp && data.base_resp.ret !== 0) {
    const errorMsg = `业务错误: ${data.base_err_msg || data.base_resp.err_msg || 'Unknown error'}`;
    console.error(`✗ ${errorMsg}`);

    // 检查是否是业务层面的认证失败
    const 业务错误码 = data.base_resp.ret;
    if (业务错误码 === 20003 || 业务错误码 === 20002) { // 假设这些是认证错误码
      console.log('⚠  检测到会话过期，尝试重新认证...');

      await clearSession();
      const success = await relogin();
      if (!success) {
        throw new Error('会话过期，请重新登录');
      }

      console.log('🔄 重试请求...');
      return request(url, options, retryCount + 1);
    }

    throw new Error(errorMsg);
  }

  return data;
}

/**
 * 保存登录密钥到会话（如果需要）
 */
async function saveLoginKey(authKey) {
  await saveSession(authKey);
}

/**
 * 导出函数供其他脚本使用
 */
export {
  request,
  getAuthKey,
  saveLoginKey,
  clearSession,
  isSessionExpired,
  loadSession,
  saveSession,
};

/**
 * 如果直接运行此脚本，显示当前会话状态
 */
async function main() {
  console.log('=== 登录会话状态检查 ===\n');

  const session = await loadSession();
  const authKey = await getAuthKey();

  if (session && !isSessionExpired(session)) {
    const ageInHours = ((Date.now() - session.createdAt) / (1000 * 60 * 60)).toFixed(1);
    console.log(`✓ 会话有效`);
    console.log(`  创建时间: ${new Date(session.createdAt).toLocaleString('zh-CN')}`);
    console.log(`  距现在: ${ageInHours} 小时`);
    console.log(`  距过期: ${(AUTH_KEY_EXPIRY_DAYS * 24 - ageInHours).toFixed(1)} 小时`);
  } else {
    console.log(`⚠ 无有效会话`);
    console.log(`  使用默认密钥: ${authKey.substring(0, 8)}...`);
  }

  // 测试请求
  console.log('\n测试 API 连接...');
  try {
    const data = await request(`${BASE_URL}/api/web/mp/accounts`, { headers: {} });
    console.log(`✓ API 连接成功，共 ${data.list.length} 个公众号`);
  } catch (error) {
    console.error(`✗ API 连接失败: ${error.message}`);
    console.log('\n建议：');
    console.log('1. 检查微信文章导出器是否正在运行');
    console.log('2. 检查端口 3200 是否正确');
    console.log('3. 尝试重新获取登录密钥');
  }
}

main().catch(console.error);
