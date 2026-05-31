#!/usr/bin/env node

/**
 * 大模型分析模块
 * 支持多种LLM API进行新闻分析
 */

/**
 * 分析单篇文章，提取其中的多条新闻
 * @param {string} title - 文章标题
 * @param {string} content - 文章内容
 * @param {object} config - 配置选项
 * @param {string} config.apiKey - API密钥
 * @param {string} config.model - 模型名称（如 gpt-4, claude-3-sonnet）
 * @param {string} config.baseUrl - API基础URL
 * @param {string} config.provider - 提供商（openai, anthropic, qwen等）
 */
async function analyzeArticleWithLLM(title, content, config = {}) {
  const {
    apiKey = '',
    model = 'gpt-4-mini',
    baseUrl = 'https://api.openai.com/v1',
    provider = 'openai',
    industryName = '行业',
    industryKeywords = [],
    industryCategories = []
  } = config;

  if (!apiKey) {
    throw new Error('请提供API密钥');
  }

  const maxLength = 15000;
  const truncatedContent = content ? content.substring(0, maxLength) : '';

  const systemPrompt = buildSystemPrompt(industryName, industryKeywords, industryCategories);

  const prompt = ANALYSIS_PROMPT
    .replace('{content}', truncatedContent);

  try {
    let result;

    if (provider === 'anthropic' || baseUrl.includes('anthropic')) {
      result = await callAnthropic(prompt, apiKey, model, baseUrl, systemPrompt);
    } else {
      result = await callOpenAI(prompt, apiKey, model, baseUrl, provider, systemPrompt);
    }

    return parseAnalysisResult(result);
  } catch (error) {
    console.error(`大模型分析失败: ${error.message}`);
    return {
      has_news: false,
      news_list: [],
      analysis_summary: `分析失败: ${error.message}`
    };
  }
}

/**
 * OpenAI API调用
 */
async function callOpenAI(prompt, apiKey, model, baseUrl, provider = 'openai', systemPrompt = SYSTEM_PROMPT) {
  const url = `${baseUrl}/chat/completions`;

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${apiKey}`
    },
    body: JSON.stringify({
      model: model,
      messages: [
        {
          role: 'system',
          content: systemPrompt
        },
        {
          role: 'user',
          content: prompt
        }
      ],
      temperature: 0.1,
      max_tokens: 4000
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API请求失败: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.choices[0].message.content;
}

/**
 * Anthropic Claude API调用
 */
async function callAnthropic(prompt, apiKey, model, baseUrl, systemPrompt = SYSTEM_PROMPT) {
  const response = await fetch(`${baseUrl}/v1/messages`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': apiKey,
      'anthropic-version': '2023-06-01'
    },
    body: JSON.stringify({
      model: model,
      max_tokens: 4000,
      system: systemPrompt,
      messages: [
        {
          role: 'user',
          content: prompt
        }
      ]
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API请求失败: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.content[0].text;
}

/**
 * 解析大模型分析结果
 */
function parseAnalysisResult(result) {
  try {
    // 尝试提取JSON
    const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```/);
    let jsonStr = null;

    if (jsonMatch) {
      jsonStr = jsonMatch[1];
    } else {
      // 尝试直接解析
      const startIdx = result.indexOf('{');
      const endIdx = result.lastIndexOf('}');
      if (startIdx !== -1 && endIdx !== -1) {
        jsonStr = result.substring(startIdx, endIdx + 1);
      }
    }

    if (jsonStr) {
      const parsed = JSON.parse(jsonStr);

      // 验证格式
      if (!parsed.hasOwnProperty('has_news')) {
        console.warn('返回JSON缺少has_news字段');
        parsed.has_news = false;
      }

      if (parsed.has_news && Array.isArray(parsed.news_list)) {
        console.log(`  ✓ 提取到 ${parsed.news_list.length} 条新闻`);
      }

      return parsed;
    }

    // 解析失败
    console.warn('无法解析JSON返回');
    return {
      has_news: false,
      news_list: [],
      analysis_summary: '无法解析分析结果'
    };
  } catch (error) {
    console.error(`解析JSON失败: ${error.message}`);
    console.error('原始结果:', result.substring(0, 500));
    return {
      has_news: false,
      news_list: [],
      analysis_summary: `解析失败: ${error.message}`
    };
  }
}

/**
 * 系统提示词 - 用户提供的格式
 */
function buildSystemPrompt(industryName, industryKeywords, industryCategories) {
  const keywordsStr = industryKeywords.length > 0
    ? industryKeywords.join('、')
    : '相关行业';

  const categoryLines = industryCategories.length > 0
    ? industryCategories.map((c, i) => `${i + 1}. **${c.name}**：${c.description}`).join('\n')
    : '1. **行业动态**：相关行业新闻';

  const categoryNames = industryCategories.length > 0
    ? industryCategories.map(c => c.name).join('、')
    : '行业动态';

  return `你是一个专业的新闻分析助手，专门分析${industryName}相关的新闻。

## 目标行业范围

${categoryLines}

## 关键词参考

${keywordsStr}

## 剔除标准（严格）

必须剔除的内容：
1. **招聘/人才信息**：招聘启事、岗位需求、人才流动、人员任用、薪资福利等
2. **访谈/人物故事**：人物采访、高管访谈、创业故事、个人经历分享等（除非访谈内容包含重要的业务信息）
3. **其他不相关行业**：与${industryName}无关的行业新闻

## 新闻类型

融资投资、政策法规、市场动态、技术创新、财务报告、战略合作、会展信息、项目动态

## 特殊规则

- 如果内容涉及多个行业，**必须至少有一个核心目标行业**才能收录
- 优先收录与${industryName}直接相关的内容
- 审慎判断，宁可漏录也不要收录无关内容

返回JSON格式：
\`\`\`json
{
  "has_news": true,
  "news_list": [
    {
      "title": "新闻标题",
      "summary": "详细摘要",
      "industry_type": "行业类型（必须从以下选择：${categoryNames}）",
      "news_type": "新闻类型",
      "confidence": 0.8
    }
  ],
  "analysis_summary": "分析完成"
}
\`\`\`

如果没有符合条件的新闻，返回：
\`\`\`json
{
  "has_news": false,
  "news_list": [],
  "analysis_summary": "文章内容不符合目标行业范围"
}
\`\`\``;
}

const SYSTEM_PROMPT = buildSystemPrompt('算力，数据中心，AI', ['算力', '数据中心', 'AI', '人工智能', 'GPU', '芯片', '智算', '液冷', '服务器', '云计算', '大模型'], [
  { name: '数据中心', description: '数据中心建设、IDC机房、服务器部署、数据中心基础设施、能源供应、算力园区、数字基础设施等' },
  { name: '算力', description: 'AI计算芯片（GPU、ASIC、TPU等）、算力租赁、算力调度、算力网络、高性能计算、AI芯片采购、芯片制造产业链、存储芯片等' },
  { name: '云计算', description: '云服务、公有云、私有云、混合云、云基础设施、云原生技术、云数据库等' },
  { name: '人工智能', description: '大语言模型、AI算法、深度学习、机器学习、AI框架、AI应用、AI硬件、AI算力需求、AI商业化等' },
  { name: '大数据', description: '数据中台、数据治理、数据分析、数据湖、数据仓库、商业智能等' },
  { name: '跨境数据', description: '数据跨境流动、数据出境、国际数据合作、数据主权等' }
]);

/**
 * 分析提示词模板 - 用户提供的格式
 */
const ANALYSIS_PROMPT = `分析以下文章内容，提取新闻信息：

{content}`;

export {
  analyzeArticleWithLLM,
  callOpenAI,
  callAnthropic,
  parseAnalysisResult,
  buildSystemPrompt,
  SYSTEM_PROMPT,
  ANALYSIS_PROMPT
};

// 如果直接运行此脚本
if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('大模型分析模块已加载');
  console.log('使用方法：');
  console.log('  analyzeArticleWithLLM(title, content, config)');
}
