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
    provider = 'openai'
  } = config;

  if (!apiKey) {
    throw new Error('请提供API密钥');
  }

  // 限制内容长度（可以给更多内容让LLM分析）
  const maxLength = 15000;
  const truncatedContent = content ? content.substring(0, maxLength) : '';

  const prompt = ANALYSIS_PROMPT
    .replace('{content}', truncatedContent);

  try {
    let result;

    if (provider === 'openai' || baseUrl.includes('openai')) {
      result = await callOpenAI(prompt, apiKey, model, baseUrl, 'openai');
    } else if (provider === 'anthropic' || baseUrl.includes('anthropic')) {
      result = await callAnthropic(prompt, apiKey, model, baseUrl);
    } else if (provider === 'yovole' || baseUrl.includes('yovole')) {
      result = await callOpenAI(prompt, apiKey, model, baseUrl, 'yovole');
    } else {
      result = await callOpenAI(prompt, apiKey, model, baseUrl, provider);
    }

    // 解析结果
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
async function callOpenAI(prompt, apiKey, model, baseUrl, provider = 'openai') {
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
          content: SYSTEM_PROMPT
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
async function callAnthropic(prompt, apiKey, model, baseUrl) {
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
      system: SYSTEM_PROMPT,
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
const SYSTEM_PROMPT = `你是一个专业的新闻分析助手，专门分析数据中心、算力、云计算、人工智能、大数据、跨境数据等六个行业的新闻。

## 目标行业范围（仅限以下6类）

1. **数据中心**：数据中心建设、IDC机房、服务器部署、数据中心基础设施、能源供应、算力园区、数字基础设施等
2. **算力**：AI计算芯片（GPU、ASIC、TPU等）、算力租赁、算力调度、算力网络、高性能计算、AI芯片采购、芯片制造产业链、存储芯片等
3. **云计算**：云服务、公有云、私有云、混合云、云基础设施、云原生技术、云数据库等
4. **人工智能**：大语言模型、AI算法、深度学习、机器学习、AI框架、AI应用、AI硬件、AI算力需求、AI商业化等（注意：通用机器人、脑机接口、自动驾驶等单独的AI应用不属于本类型）
5. **大数据**：数据中台、数据治理、数据分析、数据湖、数据仓库、商业智能等
6. **跨境数据**：数据跨境流动、数据出境、国际数据合作、数据主权等

## 剔除标准（严格）

必须剔除的内容：
1. **招聘/人才信息**：招聘启事、岗位需求、人才流动、人员任用、薪资福利等（即使招聘岗位在目标行业）
2. **访谈/人物故事**：人物采访、高管访谈、创业故事、个人经历分享等（除非访谈内容包含重要的业务信息）
3. **商业航天**：火箭发射、卫星、太空旅游等（即使涉及AI技术）
4. **交通运输**：航运、无人船、自动驾驶出行服务等
5. **新能源**：光伏、储能、新能源电池等（除非专门服务于数据中心）
6. **生物科技/医疗**：生物合成、医疗器械、医药等（除非是AI医疗大模型）
7. **消费电子**：手机、家电等（除非是AI芯片相关）
8. **通用机器人行业**：人形机器人统计、机器人职称等（除非明确是AI机器人技术突破）
9. **脑机接口/神经科学**：Neuralink等脑机接口产品
10. **汽车行业**：特斯拉车型、自动驾驶等（除非是AI芯片或算力中心）
11. **其他完全不相关行业**：餐饮、教育、游戏、体育、地缘政治等

## 新闻类型

融资投资、政策法规、市场动态、技术创新、财务报告、战略合作、会展信息、项目动态

## 特殊规则

- 如果内容涉及多个行业，**必须至少有一个核心目标行业**才能收录
- 如果仅仅是某个其他行业的"数字化"或"智能化"，不属于本类型（如：航运的智能化、新能源的数字化）
- 优先收录与算力基础设施、AI大模型、数据中心直接相关的内容
- 审慎判断，宁可漏录也不要收录无关内容

返回JSON格式：
\`\`\`json
{
  "has_news": true,
  "news_list": [
    {
      "title": "新闻标题",
      "summary": "详细摘要",
      "industry_type": "行业类型（必须从上述6类中选择）",
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
  SYSTEM_PROMPT,
  ANALYSIS_PROMPT
};

// 如果直接运行此脚本
if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('大模型分析模块已加载');
  console.log('使用方法：');
  console.log('  analyzeArticleWithLLM(title, content, config)');
}
