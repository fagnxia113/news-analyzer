#!/usr/bin/env node

/**
 * 分析昨天18点至今天18点的新闻（简化版）
 *
 * 新版格式：
 * 1. 🔥 今日重点（AI筛选，最多3条）
 * 2. 分类新闻（融资、数据中心、技术、风险、其他）
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
  const args = process.argv.slice(2);

  if (args.length >= 2) {
    const startTime = new Date(args[0]);
    const endTime = new Date(args[1]);

    if (isNaN(startTime.getTime()) || isNaN(endTime.getTime())) {
      console.error('❌ 时间格式错误，请使用格式: "YYYY-MM-DD HH:mm"');
      process.exit(1);
    }

    return {
      START_TIME: Math.floor(startTime.getTime() / 1000),
      END_TIME: Math.floor(endTime.getTime() / 1000)
    };
  }

  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);

  const yesterday18 = new Date(yesterday);
  yesterday18.setHours(18, 0, 0, 0);

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
    return [];
  }

  const publishPage = JSON.parse(response.publish_page);
  return publishPage.publish_list.flatMap(item => {
    const publishInfo = JSON.parse(item.publish_info);
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
    return await response.text();
  } catch (error) {
    console.error(`  警告: 无法获取文章内容 - ${error.message}`);
    return null;
  }
}

async function deduplicateNewsWithLLM(newsList, llmConfig) {
  if (newsList.length <= 1) {
    return newsList.map(n => ({ ...n, sources: [n.account] }));
  }

  console.log(`  🤖 使用LLM去重...`);

  try {
    const newsText = newsList.map((n, i) => {
      return `${i + 1}. [${n.industry_type}] ${n.title}\n    来源: ${n.account}\n    内容: ${n.summary.substring(0, 200)}...`;
    }).join('\n\n');

    const prompt = `分析以下${newsList.length}条新闻，严格识别重复新闻。

新闻列表：
${newsText}

判断标准：
1. 同一事件的不同报道 = 重复
2. 同一公司同一动作 = 重复
3. 同一产品/项目的报道 = 重复
4. 相似但不完全相同的不算重复

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "duplicate_groups": [
    {
      "indices": [1, 3, 5],
      "keep": 1
    }
  ]
}
\`\`\`
若无重复返回{"duplicate_groups":[]}。`;

    const result = await callOpenAI(
      prompt,
      llmConfig.apiKey,
      llmConfig.model,
      llmConfig.baseUrl,
      llmConfig.provider
    );

    const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```\s*/) ||
                    result.match(/\{[\s\S]*\}/);

    if (!jsonMatch) {
      return newsList.map(n => ({ ...n, sources: [n.account] }));
    }

    const jsonStr = jsonMatch[1] || jsonMatch[0];
    const parsed = JSON.parse(jsonStr);

    if (!parsed.duplicate_groups || !Array.isArray(parsed.duplicate_groups)) {
      return newsList.map(n => ({ ...n, sources: [n.account] }));
    }

    const duplicateMap = new Map();
    parsed.duplicate_groups.forEach(group => {
      const keepIdx = group.keep - 1;
      const indices = group.indices.map(i => i - 1);
      indices.forEach(idx => {
        if (idx !== keepIdx) {
          duplicateMap.set(idx, keepIdx);
        }
      });
      console.log(`    - 识别重复: [${indices.map(x => x + 1).join(', ')}] 归并保留 ${keepIdx + 1}`);
    });

    const deduplicated = [];
    const sourcesMap = new Map();

    newsList.forEach((n, i) => {
      sourcesMap.set(i, [n.account]);
    });

    duplicateMap.forEach((targetIdx, sourceIdx) => {
      const sourceAccounts = sourcesMap.get(sourceIdx) || [];
      const targetAccounts = sourcesMap.get(targetIdx) || [];
      const merged = new Set([...sourceAccounts, ...targetAccounts]);
      sourcesMap.set(targetIdx, Array.from(merged));
    });

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
    console.log(`  ✓ 去重完成: 剔除 ${removedCount} 条，保留 ${deduplicated.length} 条`);

    return deduplicated;

  } catch (error) {
    console.error(`  ❌ LLM去重失败: ${error.message}`);
    return newsList.map(n => ({ ...n, sources: [n.account] }));
  }
}

async function processSingleArticle(article, index, total) {
  console.log(`\n[${index}/${total}] ${article.title.substring(0, 60)}...`);

  try {
    console.log('   正在获取内容...');
    const content = await getArticleContent(article.link);

    if (!content) {
      console.log('   ✗ 无法获取文章内容，跳过');
      return [];
    }

    if (USE_LLM && llmConfig) {
      console.log('   正在使用LLM分析...');
      const analysisResult = await analyzeArticleWithLLM(article.title, content, llmConfig);

      if (analysisResult.has_news && Array.isArray(analysisResult.news_list)) {
        const newsList = analysisResult.news_list.map(news => ({
          title: news.title,
          summary: news.summary,
          industry_type: news.industry_type,
          news_type: news.news_type,
          link: article.link,
          account: article.account,
          createTime: article.createTime
        }));
        console.log(`   ✓ 提取到 ${newsList.length} 条新闻`);
        return newsList;
      } else {
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
 * 使用AI筛选今日重点新闻（最多3条）
 */
async function selectTopNews(allNews, llmConfig) {
  if (!llmConfig || allNews.length === 0) {
    return [];
  }

  console.log('\n🔍 AI正在筛选今日重点新闻...');

  const newsSummaries = allNews.map((n, i) =>
    `${i + 1}. ${n.title}\n   ${n.summary.substring(0, 150)}...`
  ).join('\n\n');

  const prompt = `分析以下${allNews.length}条新闻，选出**最多3条**对行业最重要、最有价值的新闻。

判断标准（从多维度综合考量，不是按金额大小）：
1. 行业影响：这件事会不会改变行业格局/趋势？
2. 里程碑事件：是不是"首次"、"突破"、"首个"这类里程碑？
3. 战略意义：对产业链上下游有什么影响？
4. 趋势信号：反映了什么重要的行业变化？
5. 影响范围：影响的是特定公司还是整个行业？

不要只看融资额大小。例如：
- 一家小公司突破某技术瓶颈，可能比大额融资更重要
- 某政策变化，可能影响整个行业
- 某危机事件，可能揭示行业风险

新闻列表：
${newsSummaries}

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "top_news": [
    {
      "index": 1,
      "reason": "简述为什么这条新闻重要（1-2句话）",
      "insight": "这条新闻意味着什么（1-2句话，给从业者什么启示）"
    }
  ]
}
\`\`\`
index使用上述列表的实际编号（1到${allNews.length}）。`;

  try {
    const result = await callOpenAI(
      prompt,
      llmConfig.apiKey,
      llmConfig.model,
      llmConfig.baseUrl,
      llmConfig.provider
    );

    const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```\s*/) ||
                      result.match(/\{[\s\S]*\}/);

    if (!jsonMatch) {
      console.log('   ⚠ 无法解析AI返回结果');
      return [];
    }

    const jsonStr = jsonMatch[1] || jsonMatch[0];
    const parsed = JSON.parse(jsonStr);

    if (!parsed.top_news || !Array.isArray(parsed.top_news)) {
      console.log('   ✓ 未发现特别重要的新闻');
      return [];
    }

    const topNewsList = [];
    parsed.top_news.forEach(item => {
      const idx = item.index - 1;
      if (idx >= 0 && idx < allNews.length) {
        topNewsList.push({
          news: allNews[idx],
          reason: item.reason,
          insight: item.insight
        });
      }
    });

    console.log(`   ✓ 筛选出 ${topNewsList.length} 条重点新闻`);
    return topNewsList;

  } catch (error) {
    console.error(`   ❌ 筛选失败: ${error.message}`);
    return [];
  }
}

/**
 * 使用AI自动分类新闻
 */
async function categorizeNews(allNews, llmConfig) {
  if (!llmConfig || allNews.length === 0) {
    return { '其他': allNews };
  }

  console.log('\n📂 AI正在自动分类新闻...');

  const newsList = allNews.map((n, i) =>
    `${i + 1}. ${n.title}\n   ${n.summary.substring(0, 100)}...`
  ).join('\n\n');

  const prompt = `将以下${allNews.length}条新闻分类到以下类别中：
- 融资与投资：融资、投资、收购、上市、发债等
- 数据中心建设：数据中心新建、扩建、选址、项目获批等
- 技术动态：技术发布、产品发布、模型发布、技术突破等
- 风险：安全事件、监管风险、纠纷诉讼、预警等
- 其他：不符合以上任何类别的新闻

新闻列表：
${newsList}

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "categories": {
    "融资与投资": [1, 3, 5],
    "数据中心建设": [2, 4],
    "技术动态": [6, 7, 8],
    "风险": [],
    "其他": []
  }
}
\`\`\`
index使用上述列表的实际编号（1到${allNews.length}）。每条新闻只能属于一个类别。`;

  try {
    const result = await callOpenAI(
      prompt,
      llmConfig.apiKey,
      llmConfig.model,
      llmConfig.baseUrl,
      llmConfig.provider
    );

    const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```\s*/) ||
                      result.match(/\{[\s\S]*\}/);

    if (!jsonMatch) {
      console.log('   ⚠ 无法解析AI返回结果，归入"其他"');
      return { '其他': allNews };
    }

    const jsonStr = jsonMatch[1] || jsonMatch[0];
    const parsed = JSON.parse(jsonStr);

    if (!parsed.categories || typeof parsed.categories !== 'object') {
      return { '其他': allNews };
    }

    const categorized = {};
    const emojiMap = {
      '融资与投资': '💰',
      '数据中心建设': '🏢',
      '技术动态': '🔧',
      '风险': '⚠️',
      '其他': '📊'
    };

    for (const [category, indices] of Object.entries(parsed.categories)) {
      if (Array.isArray(indices) && indices.length > 0) {
        const key = emojiMap[category] ? category : '其他';
        categorized[key] = indices.map(idx => allNews[idx - 1]).filter(n => n);
      }
    }

    const categorizedNews = new Set();
    Object.values(categorized).forEach(list => list.forEach(n => categorizedNews.add(n)));

    if (categorizedNews.size < allNews.length) {
      const uncategorized = allNews.filter(n => !categorizedNews.has(n));
      if (!categorized['其他']) categorized['其他'] = [];
      categorized['其他'].push(...uncategorized);
    }

    const totalCount = Object.values(categorized).reduce((sum, list) => sum + list.length, 0);
    console.log(`   ✓ 分类完成: ${totalCount} 条`);

    return categorized;

  } catch (error) {
    console.error(`   ❌ 分类失败: ${error.message}`);
    return { '其他': allNews };
  }
}

/**
 * 生成Markdown报告（简化版）
 */
async function generateMarkdownReport(startDate, endDate, allNews, topNewsList, categorizedNews) {
  const today = new Date();
  const dateStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;

  let report = `# 算力、数据中心、AI动态日报 | ${dateStr}\n\n`;

  // 今日重点
  if (topNewsList.length > 0) {
    report += `## 🔥 今日重点（AI筛选）\n\n`;
    topNewsList.forEach((item, idx) => {
      const news = item.news;
      report += `### ${idx + 1}. ${news.title}\n\n`;
      if (news.link) {
        report += `[原文链接](${news.link})\n\n`;
      }
      report += `${news.summary}\n\n`;
      report += `**为什么重要**：${item.reason}\n\n`;
      report += `**这意味着**：${item.insight}\n\n`;
    });
    report += `---\n\n`;
  }

  // 各分类新闻
  const categories = [
    { key: '💰 融资与投资', name: '融资与投资' },
    { key: '🏢 数据中心建设', name: '数据中心建设' },
    { key: '🔧 技术动态', name: '技术动态' },
    { key: '⚠️ 风险', name: '风险' },
    { key: '📊 其他', name: '其他' }
  ];

  for (const cat of categories) {
    const news = categorizedNews[cat.name];
    if (news && news.length > 0) {
      report += `## ${cat.key}\n\n`;

      // 今日重点中的新闻不重复显示
      const topNewsTitles = new Set(topNewsList.map(n => n.news.title));

      for (const item of news) {
        if (topNewsTitles.has(item.title)) {
          continue;
        }

        report += `### ${item.title}\n\n`;
        if (item.link) {
          report += `[原文链接](${item.link})\n\n`;
        }
        report += `${item.summary}\n\n`;
      }
    }
  }

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

  console.log('=== 新闻分析报告（简化版）===');
  console.log(`时间范围: ${formatDate(startDate)} 至 ${formatDate(endDate)}`);

  const accounts = await getAccounts();
  console.log(`共有 ${accounts.length} 个公众号\n`);

  if (USE_LLM && llmConfig) {
    console.log(`LLM配置: ${llmConfig.provider} / ${llmConfig.model || 'default'}\n`);
  }

  if (!USE_LLM) {
    console.log('❌ 未配置LLM，请配置.env文件');
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

  // 阶段2: 分析文章
  console.log('\n阶段 2/5: 分析文章...');
  console.log('='.repeat(60));

  const allNews = [];
  for (let i = 0; i < allArticles.length; i++) {
    const newsList = await processSingleArticle(allArticles[i], i + 1, allArticles.length);
    if (newsList.length > 0) {
      allNews.push(...newsList);
    }
    if (i < allArticles.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }

  console.log('\n' + '='.repeat(60));
  console.log(`\n阶段 3/5: 分析完成`);
  console.log(`  提取新闻: ${allNews.length} 条`);

  // 阶段4: 智能去重
  console.log('\n阶段 4/5: 智能去重...');
  const deduplicatedNews = await deduplicateNewsWithLLM(allNews, llmConfig);
  console.log(`  ✓ 去重完成: 保留 ${deduplicatedNews.length} 条`);

  // 阶段5: AI筛选重点新闻和分类
  console.log('\n阶段 5/5: AI筛选和分类...');

  // 筛选重点新闻
  const topNewsList = await selectTopNews(deduplicatedNews, llmConfig);

  // 自动分类
  const categorizedNews = await categorizeNews(deduplicatedNews, llmConfig);

  // 生成报告
  const report = await generateMarkdownReport(
    formatDate(startDate),
    formatDate(endDate),
    deduplicatedNews,
    topNewsList,
    categorizedNews
  );

  // 保存文件
  const fs = await import('fs');
  const outputDir = 'd:\\业余兴趣\\新闻分析结果';
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  const now = new Date();
  const dateStr = `${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`;
  const mdPath = `${outputDir}\\算力、数据中心、AI动态日报${dateStr}.md`;
  fs.writeFileSync(mdPath, report, 'utf-8');

  console.log(`\n报告已保存: ${mdPath}`);
  console.log('\n=== 完成 ===');
}

main().catch(error => {
  console.error('程序出错:', error);
  process.exit(1);
});
