#!/usr/bin/env node

/**
 * 分析昨天18点至今天18点的新闻（LLM版本）
 */

import { request } from './auth-manager.js';
import { analyzeArticleWithLLM, callOpenAI } from './llm-analyzer.js';
import { getLLMConfig } from './config-manager.js';

const BASE_URL = 'http://127.0.0.1:3200';

// 获取 LLM 配置
const llmConfig = await getLLMConfig();

// 是否使用 LLM 分析
const USE_LLM = !!llmConfig;

// 计算时间范围：支持自定义时间或默认使用昨天18点 - 今天18点
function getTimeRange() {
  // 检查命令行参数
  const args = process.argv.slice(2);

  if (args.length >= 2) {
    // 从命令行参数读取时间范围
    // 格式: node daily-analysis-llm.js "2026-02-15 18:00" "2026-02-24 18:00"
    const startTime = new Date(args[0]);
    const endTime = new Date(args[1]);

    if (isNaN(startTime.getTime()) || isNaN(endTime.getTime())) {
      console.error('❌ 时间格式错误，请使用格式: "YYYY-MM-DD HH:mm"');
      console.error('示例: node daily-analysis-llm.js "2026-02-15 18:00" "2026-02-24 18:00"');
      process.exit(1);
    }

    return {
      START_TIME: Math.floor(startTime.getTime() / 1000),
      END_TIME: Math.floor(endTime.getTime() / 1000)
    };
  }

  // 默认：昨天18点 - 今天18点
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);

  // 昨天18点
  const yesterday18 = new Date(yesterday);
  yesterday18.setHours(18, 0, 0, 0);

  // 今天18点
  const today18 = new Date(today);
  today18.setHours(18, 0, 0, 0);

  return {
    START_TIME: Math.floor(yesterday18.getTime() / 1000),
    END_TIME: Math.floor(today18.getTime() / 1000)
  };
}

const { START_TIME, END_TIME } = getTimeRange();

async function getAccounts() {
  const response = await request(`${BASE_URL}/api/web/mp/accounts`);
  return response.list;
}

async function getArticles(fakeid, size = 200) {
  const response = await request(
    `${BASE_URL}/api/web/mp/appmsgpublish?id=${fakeid}&begin=0&size=${size}`
  );

  if (response.base_resp.ret !== 0) {
    console.error(`Error fetching articles for ${fakeid}: ${response.base_resp.err_msg}`);
    return [];
  }

  const publishPage = JSON.parse(response.publish_page);
  return publishPage.publish_list.flatMap(item => {
    const publishInfo = JSON.parse(item.publish_info);
    // 使用 sent_info.time 作为实际发布时间，如果没有则使用 create_time
    const publishTime = publishInfo.sent_info?.time;
    return publishInfo.appmsgex.map(appmsg => ({
      ...appmsg,
      publish_time: publishTime || appmsg.create_time
    }));
  });
}

async function getArticleContent(link) {
  try {
    const response = await fetch(`${BASE_URL}/api/public/v1/download?url=${encodeURIComponent(link)}&format=text`);
    if (!response.ok) {
      throw new Error(`Failed to fetch: ${response.status}`);
    }
    const text = await response.text();
    return text;
  } catch (error) {
    console.error(`  警告: 无法获取文章内容 - ${error.message}`);
    return null;
  }
}

/**
 * 使用LLM智能去重新闻列表
 * 基于语义分析识别重复新闻
 */
async function deduplicateNewsWithLLM(newsList, llmConfig) {
  if (newsList.length <= 1) {
    return newsList.map(n => ({ ...n, sources: [n.account] }));
  }

  console.log(`  🤖 使用LLM统一去重分析...`);

  try {
    // 构建完整的新闻列表（展示所有新闻）
    const newsText = newsList.map((n, i) => {
      return `${i + 1}. [${n.industry_type}] ${n.title}\n    来源: ${n.account}\n    内容: ${n.summary.substring(0, 200)}...`;
    }).join('\n\n');

    const prompt = `分析以下${newsList.length}条新闻，严格识别重复新闻。

新闻列表：
${newsText}

判断标准（非常严格，宁可漏删也不误删）：
1. 同一事件的不同报道（不同媒体/不同角度）= 重复
2. 同一公司同一动作（如同一轮融资、同一收购、同一产品发布）= 重复
3. 同一产品/项目/模型/芯片的不同角度报道 = 重复
4. 数据或时间略有不同，但本质是同一事件 = 重复
5. 同一政策/法规的不同解读 = 重复
6. 同一财报/业绩数据的不同呈现 = 重复

注意事项：
- 相似但不完全相同的新闻（如同一公司做不同的事）不算是重复
- 不同公司的新闻不算是重复
- 只标记明确重复的新闻，存疑的保留

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "duplicate_groups": [
    {
      "indices": [1, 3, 5],
      "keep": 1,
      "reason": "简述原因"
    }
  ]
}
\`\`\`
indices使用上述列表的实际编号（1到${newsList.length}）。
keep策略：优先保留摘要更完整、发布时间更早者。
若无重复返回{"duplicate_groups":[]}。`;

    const result = await callOpenAI(
      prompt,
      llmConfig.apiKey,
      llmConfig.model,
      llmConfig.baseUrl,
      llmConfig.provider
    );

    const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```/) ||
                    result.match(/{[\s\S]*}/);

    if (!jsonMatch) {
      console.log('  ⚠ 无法解析LLM返回结果，保留全部');
      return newsList.map(n => ({ ...n, sources: [n.account] }));
    }

    const jsonStr = jsonMatch[1] || jsonMatch[0];
    const parsed = JSON.parse(jsonStr);

    if (!parsed.duplicate_groups || !Array.isArray(parsed.duplicate_groups)) {
      console.log(`  ✓ 未发现重复新闻`);
      return newsList.map(n => ({ ...n, sources: [n.account] }));
    }

    // 构建去重映射
    const duplicateMap = new Map(); // 索引-> 要保留的索引
    parsed.duplicate_groups.forEach(group => {
      const keepIdx = group.keep - 1; // 转换为0-based
      const indices = group.indices.map(i => i - 1);
      indices.forEach(idx => {
        if (idx !== keepIdx) {
          duplicateMap.set(idx, keepIdx);
        }
      });
      console.log(`    - 识别重复: [${indices.map(x => x + 1).join(', ')}] 归并保留 ${keepIdx + 1}`);
    });

    // 构建去重后的列表，合并来源
    const deduplicated = [];
    const sourcesMap = new Map();

    // 初始化来源映射
    newsList.forEach((n, i) => {
      sourcesMap.set(i, [n.account]);
    });

    // 合并重复新闻的来源
    duplicateMap.forEach((targetIdx, sourceIdx) => {
      const sourceAccounts = sourcesMap.get(sourceIdx) || [];
      const targetAccounts = sourcesMap.get(targetIdx) || [];
      const merged = new Set([...sourceAccounts, ...targetAccounts]);
      sourcesMap.set(targetIdx, Array.from(merged));
    });

    // 只保留未被标记为重复的索引
    const allIndices = new Set([...Array(newsList.length).keys()]);
    const duplicateIndices = new Set(duplicateMap.keys());

    allIndices.forEach(idx => {
      if (!duplicateIndices.has(idx)) {
        const news = {
          ...newsList[idx],
          sources: sourcesMap.get(idx) || [newsList[idx].account]
        };
        if (news.sources.length > 1) {
          news.summary = `${news.summary} [来源: ${news.sources.join(', ')}]`;
        }
        deduplicated.push(news);
      }
    });

    const removedCount = newsList.length - deduplicated.length;
    console.log(`  ✓ 去重完成: 剔除 ${removedCount} 条重复，保留 ${deduplicated.length} 条`);

    return deduplicated;

  } catch (error) {
    console.error(`  ❌ LLM去重失败: ${error.message}`);
    console.log('  🔄 降级到字符串相似度去重');
    return deduplicateNewsBySimilarity(newsList);
  }
}

/**
 * 字符串相似度去重（备用方案）
 */
function deduplicateNewsBySimilarity(newsList) {
  const threshold = 0.6;
  const deduplicated = [];
  const seen = [];

  for (const news of newsList) {
    let isDuplicate = false;

    for (const existing of seen) {
      const similarity = calculateSimilarity(news.title, existing.title);
      if (similarity >= threshold) {
        isDuplicate = true;
        if (news.summary && news.summary.length > (existing.summary?.length || 0)) {
          existing.summary = news.summary;
          if (!existing.sources) existing.sources = [];
          if (!existing.sources.includes(news.account)) {
            existing.sources.push(news.account);
          }
        } else {
          if (!existing.sources) existing.sources = [];
          if (!existing.sources.includes(news.account)) {
            existing.sources.push(news.account);
          }
        }
        break;
      }
    }

    if (!isDuplicate) {
      seen.push({
        title: news.title,
        summary: news.summary,
        sources: [news.account]
      });
      deduplicated.push({
        ...news,
        sources: [news.account]
      });
    }
  }

  return deduplicated;
}

/**
 * 计算字符串相似度（简单的Levenshtein距离）
 */
function calculateSimilarity(str1, str2) {
  const len1 = str1.length;
  const len2 = str2.length;
  const matrix = [];

  for (let i = 0; i <= len1; i++) {
    matrix[i] = [i];
  }

  for (let j = 0; j <= len2; j++) {
    matrix[0][j] = j;
  }

  for (let i = 1; i <= len1; i++) {
    for (let j = 1; j <= len2; j++) {
      const cost = str1[i - 1] === str2[j - 1] ? 0 : 1;
      matrix[i][j] = Math.min(
        matrix[i - 1][j] + 1,
        matrix[i][j - 1] + 1,
        matrix[i - 1][j - 1] + cost
      );
    }
  }

  const maxLen = Math.max(len1, len2);
  return 1 - matrix[len1][len2] / maxLen;
}

/**
 * 生成智能分析计划（AI驱动）
 * 使用LLM根据实际情况动态规划分析策略
 */
async function generateAnalysisPlan(allArticles, llmConfig) {
  console.log('\n🧠 AI正在规划分析策略...');

  const articlesInfo = allArticles.map((a, i) =>
    `${i + 1}. ${a.account}: "${a.title.substring(0, 50)}..."`
  ).join('\n');

  const prompt = `你是一个智能新闻分析规划助手。以下是需要分析的文章列表：

${articlesInfo}

总文章数：${allArticles.length}篇

请生成一个优化的执行计划，目标是最快、最准确地完成新闻分析和去重。

计划应包含：
1. 并行策略：哪些文章可以同时分析（因为它们来自不同来源，内容不相关）？
2. 分析顺序：哪些文章可能包含多条新闻需要优先处理？
3. 去重策略：建议如何安排去重步骤？
4. 批次划分：如何将文章分组提高效率？

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "parallel_groups": [
    {
      "group_id": 1,
      "indices": [1, 3, 5],
      "reason": "这些文章来自不同公众号，主题无关，可并行处理"
    }
  ],
  "analysis_strategy": {
    "detailed_articles": [2, 7],
    "reason": "这些文章标题暗示可能包含多条新闻，需要详细拆分"
  },
  "deduplication_timing": "after_analysis",
  "deduplication_mode": "holistic",
  "estimated_batches": 3
}
\`\`\`

说明：
- parallel_groups: 可并行处理的文章组（indices是1-based索引）
- detailed_articles: 需要重点拆分的文章索引
- deduplication_timing: "during_analysis" 或 "after_analysis"
- deduplication_mode: "batch" (批次) 或 "holistic" (整体)`;

  try {
    const result = await callOpenAI(
      prompt,
      llmConfig.apiKey,
      llmConfig.model,
      llmConfig.baseUrl,
      llmConfig.provider
    );

    const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```/) ||
                      result.match(/{[\s\S]*}/);

    if (jsonMatch) {
      const jsonStr = jsonMatch[1] || jsonMatch[0];
      const plan = JSON.parse(jsonStr);
      console.log('   ✓ AI计划生成完成');
      if (plan.parallel_groups) {
        console.log(`   - 识别${plan.parallel_groups.length}个并行处理组`);
      }
      if (plan.deduplication_mode) {
        console.log(`   - 去重模式: ${plan.deduplication_mode}`);
      }
      return plan;
    }
  } catch (error) {
    console.log(`   ⚠ 计划生成失败，使用默认策略: ${error.message}`);
  }

  // 默认计划
  return {
    parallel_groups: [{ group_id: 1, indices: allArticles.map((_, i) => i + 1) }],
    analysis_strategy: { detailed_articles: [] },
    deduplication_timing: 'after_analysis',
    deduplication_mode: 'batch',
    estimated_batches: Math.ceil(allArticles.length / 20)
  };
}

/**
 * 并行处理文章组
 */
async function processArticleGroup(articles, groupIndex, totalGroups, llmConfig) {
  console.log(`\\n📦 处理第 ${groupIndex}/${totalGroups} 组 (${articles.length} 篇)`);
  const results = [];

  for (let i = 0; i < articles.length; i++) {
    const article = articles[i];
    const newsList = await processSingleArticle(article, i + 1, articles.length);
    if (newsList.length > 0) {
      results.push(...newsList);
    }
  }

  return results;
}

/**
 * 逐条处理单篇文章（保持原有逻辑）
 */
async function processSingleArticle(article, index, total) {
  console.log(`\n[${index}/${total}] ${article.title.substring(0, 60)}...`);

  try {
    // 获取文章内容
    console.log('   正在获取内容...');
    const content = await getArticleContent(article.link);

    if (!content) {
      console.log('   ✗ 无法获取文章内容，跳过');
      return [];
    }

    // 使用LLM分析
    if (USE_LLM && llmConfig) {
      console.log('   正在使用LLM分析...');
      const analysisResult = await analyzeArticleWithLLM(article.title, content, llmConfig);

      if (analysisResult.has_news && Array.isArray(analysisResult.news_list)) {
        const newsList = analysisResult.news_list.map(news => ({
          title: news.title,
          summary: news.summary,
          industry_type: news.industry_type,
          news_type: news.news_type,
          confidence: news.confidence || 1,
          link: article.link,
          account: article.account,
          createTime: article.createTime
        }));
        console.log(`   ✓ 提取到 ${newsList.length} 条新闻`);
        return newsList;
      } else {
        console.log(`   ℹ ${analysisResult.analysis_summary || '无符合条件的新闻'}`);
        return [];
      }
    } else {
      console.log('   ✗ 未配置LLM');
      return [];
    }
  } catch (error) {
    console.error(`   ✗ 处理失败: ${error.message}`);
    return [];
  }
}

/**
 * 使用LLM生成今日要闻分析（深度分析版）
 */
async function generateDailyAnalysis(allNews, llmConfig) {
  if (!llmConfig || allNews.length === 0) {
    return '今日暂无重要新闻。';
  }

  console.log('\n💡 正在生成深度今日要闻分析...');

  // 按影响力排序：重要事件 > 大额投资 > 政策监管 > 技术突破 > 市场动态
  const priorityKeywords = {
    'high': ['索赔', '纠纷', '诉讼', '禁令', '制裁', '亿美元', '十亿', '万亿'],
    'medium': ['收购', '投资', '融资', '合作', '批准', '发布', '宣布', '建设'],
    'normal': []
  };

  allNews.forEach(news => {
    let priority = 0;
    const title = news.title.toLowerCase();

    for (const keyword of priorityKeywords.high) {
      if (title.includes(keyword.toLowerCase())) priority += 10;
    }
    for (const keyword of priorityKeywords.medium) {
      if (title.includes(keyword.toLowerCase())) priority += 5;
    }

    news.priority = priority;
  });

  allNews.sort((a, b) => (b.priority || 0) - (a.priority || 0));

  // 选择最重要的10-15条新闻
  const topNews = allNews.slice(0, Math.min(15, allNews.length));
  const newsSummary = topNews.map((n, i) =>
    `${i + 1}. 【${n.news_type || '其他'}】${n.title}\n   ${n.summary.substring(0, 120)}...`
  ).join('\n\n');

  const prompt = `作为专业行业分析师，请对以下今日重要新闻进行深度分析：

【重要新闻列表】：
${newsSummary}

请输出一份高质量的"今日要闻分析"，要求：

1. 核心事件：
   - 概括今日最重要的1-2件大事
   - 说明事件的直接原因和关键信息
   - 指出事件的行业影响和意义

2. 行业洞察：
   - 从上述新闻出发，识别3-5个核心行业动向
   - 分析新闻反映的趋势、变化和值得关注的事件
   - 避免泛泛而谈，基于具体事实进行分析

3. 商业影响：
   - 分析这些事件对行业格局、市场竞争的影响
   - 指出对不同市场参与者（头部企业、中小企业、投资者）的启示
   - 识别潜在机会和挑战

4. 展望建议：
   - 基于今日动态，提出1-2点前瞻性判断
   - 指出值得持续跟踪的方向

【风格要求】：
- 专业但不艰涩，便于行业人士快速把握要点
- 基于具体事实和事件进行分析

直接输出分析内容，不要前缀、不要编号、不要任何多余文字。`;

  try {
    const result = await callOpenAI(
      prompt,
      llmConfig.apiKey,
      llmConfig.model,
      llmConfig.baseUrl,
      llmConfig.provider
    );

    console.log('   ✓ 今日要闻分析完成');
    return result.trim();
  } catch (error) {
    console.error(`   ✗ 分析生成失败: ${error.message}`);
    // 返回基础分析
    return `今日收录多条行业动态。最受关注的是${topNews[0]?.title || '重大事件'}等行业热点，反映出行业正在经历重要变化。建议关注相关领域的后续发展。`;
  }
}

/**
 * 生成Markdown报告
 */
function generateMarkdownReport(startDate, endDate, allNews, industryCount, newsTypeCount, accountsStr, dailyAnalysis, totalSources, uniqueNews) {
  let report = '';

  // 今日要闻分析（去除LLM可能添加的标题行）
  let cleanedAnalysis = dailyAnalysis;
  // 移除开头的 **今日要闻分析** 或 ## 今日要闻分析
  cleanedAnalysis = cleanedAnalysis.replace(/^\*?\*?\*?\*?今日要闻分析\*?\*?\*?\*?\s*\n*/m, '');
  cleanedAnalysis = cleanedAnalysis.replace(/^## 今日要闻分析\s*\n*/m, '');

  report += `## 📈 今日要闻分析\n\n`;
  report += `${cleanedAnalysis}\n\n`;
  report += `---\n\n`;

  // 新闻详情
  allNews.forEach((news, idx) => {
    report += `### ${idx + 1}. ${news.title}\n\n`;
    if (news.link) {
      report += `[原文链接](${news.link})\n\n`;
    }
    report += `${news.summary}\n\n`;
    report += `---\n\n`;
  });

  return report;
}

async function main() {
  const startDate = new Date(START_TIME * 1000);
  const endDate = new Date(END_TIME * 1000);

  const formatDate = (date) => {
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hour = String(date.getHours()).padStart(2, '0');
    const minute = String(date.getMinutes()).padStart(2, '0');
    return `${month}-${day} ${hour}:${minute}`;
  };

  console.log('=== 新闻分析报告（AI智能版）===');
  console.log(`时间范围: ${formatDate(startDate)} 至 ${formatDate(endDate)}`);
  console.log(`分析方式: AI驱动智能规划`);

  const accounts = await getAccounts();
  console.log(`共有 ${accounts.length} 个公众号\n`);

  if (USE_LLM && llmConfig) {
    console.log(`LLM配置: ${llmConfig.provider} / ${llmConfig.model || 'default'}\n`);
  }

  if (!USE_LLM) {
    console.log('❌ 未配置LLM，请配置.env文件');
    console.log('   设置 USE_LLM=true 并填写 LLM_API_KEY 和 LLM_BASE_URL');
    return;
  }

  // 阶段1: 收集所有文章
  console.log('\n阶段 1/5: 收集文章列表...');
  let allArticles = [];

  for (const account of accounts) {
    const articles = await getArticles(account.fakeid);
    const recentArticles = articles.filter(a => a.publish_time >= START_TIME && a.publish_time <= END_TIME);
    console.log(`  ${account.nickname}: ${recentArticles.length} 篇`);

    for (const article of recentArticles) {
      allArticles.push({
        title: article.title,
        link: article.link,
        account: account.nickname,
        createTime: article.publish_time
      });
    }
  }

  console.log(`  ✓ 共找到 ${allArticles.length} 篇文章`);

  // 阶段2: AI生成分析计划
  const plan = await generateAnalysisPlan(allArticles, llmConfig);

  // 阶段3: 按计划执行分析
  console.log('\n阶段 3/5: 按AI计划分析文章...');
  console.log('='.repeat(60));

  const allNews = [];
  let totalArticlesProcessed = 0;
  let totalNewsExtracted = 0;

  if (plan.parallel_groups && plan.parallel_groups.length > 0) {
    for (let i = 0; i < plan.parallel_groups.length; i++) {
      const group = plan.parallel_groups[i];
      // 将1-based索引转换为0-based，并获取对应文章
      const groupArticles = group.indices.map(idx => allArticles[idx - 1]).filter(a => a);

      const newsList = await processArticleGroup(groupArticles, i + 1, plan.parallel_groups.length, llmConfig);
      if (newsList.length > 0) {
        totalArticlesProcessed += groupArticles.filter((a, idx) =>
          allArticles.indexOf(a) !== idx && newsList.length > 0
        ).length || groupArticles.length;
        totalNewsExtracted += newsList.length;
        allNews.push(...newsList);
      }

      // 组间延迟
      if (i < plan.parallel_groups.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  } else {
    // 降级到顺序处理
    for (let i = 0; i < allArticles.length; i++) {
      const newsList = await processSingleArticle(allArticles[i], i + 1, allArticles.length);
      if (newsList.length > 0) {
        totalArticlesProcessed++;
        totalNewsExtracted += newsList.length;
        allNews.push(...newsList);
      }
      if (i < allArticles.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 2000));
      }
    }
  }

  console.log('\n' + '='.repeat(60));
  console.log(`\n阶段 4/5: 分析完成统计`);
  console.log(`  处理文章: ${allArticles.length} 篇`);
  console.log(`  提取新闻: ${allNews.length} 条`);
  console.log(`  执行计划: ${plan.parallel_groups?.length || 1} 个处理组`);

  // 阶段5: 智能去重（根据计划的模式）
  console.log('\n阶段 5/5: 智能去重...');
  const deduplicatedNews = await deduplicateNewsWithLLM(allNews, llmConfig);
  const duplicateCount = allNews.length - deduplicatedNews.length;
  console.log(`  ✓ 去重完成: 剔除 ${duplicateCount} 条重复新闻，保留 ${deduplicatedNews.length} 条`);
  console.log(`  (去重模式: ${plan.deduplication_mode || 'batch'})`);

  // 统计
  const industryCount = {};
  deduplicatedNews.forEach(n => {
    if (n.industry_type) {
      industryCount[n.industry_type] = (industryCount[n.industry_type] || 0) + 1;
    }
  });

  const newsTypeCount = {};
  deduplicatedNews.forEach(n => {
    if (n.news_type) {
      newsTypeCount[n.news_type] = (newsTypeCount[n.news_type] || 0) + 1;
    }
  });

  // 生成今日要闻分析
  const dailyAnalysis = await generateDailyAnalysis(deduplicatedNews, llmConfig);

  // 生成报告
  const accountsStr = [...new Set(deduplicatedNews.map(n => n.account))].sort().join('、');
  const totalSources = allNews.length;
  const uniqueNews = deduplicatedNews.length;
  const report = generateMarkdownReport(
    formatDate(startDate),
    formatDate(endDate),
    deduplicatedNews,
    industryCount,
    newsTypeCount,
    accountsStr,
    dailyAnalysis,
    totalSources,
    uniqueNews
  );

  // 保存文件
  const fs = await import('fs');
  const outputDir = 'd:\\业余兴趣\\新闻分析结果';
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  // 生成日期格式：MMdd
  const now = new Date();
  const dateStr = `${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`;
  const mdPath = `${outputDir}\\算力、数据中心、AI动态日报${dateStr}.md`;
  fs.writeFileSync(mdPath, report, 'utf-8');

  console.log(`\n报告已保存: ${mdPath}`);
  console.log('\\n=== AI智能分析完成 ===');
}

main().catch(error => {
  console.error('程序出错:', error);
  process.exit(1);
});
