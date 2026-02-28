#!/usr/bin/env node

/**
 * 配置管理 - 简化版 (.env)
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

/**
 * .env 文件路径
 */
const ENV_FILE = path.join(fileURLToPath(new URL('.', import.meta.url)), '.env');

/**
 * 解析 .env 文件
 */
function parseEnv(content) {
  const config = {};
  const lines = content.split('\n');

  for (const line of lines) {
    const trimmed = line.trim();
    // 跳过空行和注释
    if (!trimmed || trimmed.startsWith('#')) {
      continue;
    }

    // 解析 KEY=VALUE 格式
    const match = trimmed.match(/^([^=]+)=(.*)$/);
    if (match) {
      const key = match[1].trim();
      let value = match[2].trim();

      // 移除引号
      if ((value.startsWith('"') && value.endsWith('"')) ||
          (value.startsWith("'") && value.endsWith("'"))) {
        value = value.slice(1, -1);
      }

      config[key] = value;
    }
  }

  return config;
}

/**
 * 加载 .env 配置
 */
async function loadEnv() {
  try {
    const content = await fs.readFile(ENV_FILE, 'utf-8');
    return parseEnv(content);
  } catch (error) {
    if (error.code === 'ENOENT') {
      // 文件不存在，返回空配置
      return {};
    }
    throw error;
  }
}

/**
 * 保存 .env 配置
 */
async function saveEnv(config) {
  let content = '';

  if (config.USE_LLM !== undefined) {
    content += `USE_LLM=${config.USE_LLM}\n\n`;
  }

  content += `# LLM 服务配置\n`;
  content += `# 支持: openai, deepseek, anthropic\n`;
  content += `LLM_PROVIDER=${config.LLM_PROVIDER || 'openai'}\n`;

  content += `\n# API 密钥\n`;
  content += `LLM_API_KEY=${config.LLM_API_KEY || ''}\n`;

  content += `\n# 模型名称\n`;
  content += `# OpenAI: gpt-4o-mini, gpt-4o\n`;
  content += `# DeepSeek: deepseek-chat\n`;
  content += `# Anthropic: claude-3-5-haiku-20241022\n`;
  content += `LLM_MODEL=${config.LLM_MODEL || 'gpt-4o-mini'}\n`;

  content += `\n# API 基础地址\n`;
  content += `LLM_BASE_URL=${config.LLM_BASE_URL || 'https://api.openai.com/v1'}\n`;

  await fs.writeFile(ENV_FILE, content, 'utf-8');
}

/**
 * 获取 LLM 配置
 */
async function getLLMConfig() {
  const env = await loadEnv();

  // 从环境变量读取（优先）
  const useLLM = process.env.USE_LLM === 'true' || env.USE_LLM === 'true';

  if (!useLLM || !env.LLM_API_KEY || env.LLM_API_KEY === 'your-api-key-here') {
    return null;
  }

  // 根据 provider 设置默认 baseUrl
  const provider = env.LLM_PROVIDER || 'openai';
  let baseUrl = env.LLM_BASE_URL;

  if (!baseUrl || baseUrl === 'https://api.openai.com/v1') {
    switch (provider) {
      case 'deepseek':
        baseUrl = 'https://api.deepseek.com';
        break;
      case 'anthropic':
        baseUrl = 'https://api.anthropic.com';
        break;
      default:
        baseUrl = 'https://api.openai.com/v1';
    }
  }

  return {
    enable: true,
    provider,
    apiKey: env.LLM_API_KEY,
    model: env.LLM_MODEL || 'gpt-4o-mini',
    baseUrl
  };
}

export { loadEnv, saveEnv, getLLMConfig, ENV_FILE };
