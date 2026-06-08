#!/usr/bin/env node

/**
 * 分析昨天7点至今天7点的新闻（简化版）
 *
 * 新版格式：
 * 1. 今日判断（一句话判断 + 3个变化信号）
 * 2. 今天对谁有用
 * 3. 🔥 今日主线（默认1条主线，串联2-3个证据）
 * 4. ⚡ 一分钟速览（全篇最多6-8条）
 */

import { request } from './auth-manager.js';
import { analyzeArticleWithLLM, callOpenAI } from './llm-analyzer.js';
import { getLLMConfig, getIndustryConfig } from './config-manager.js';

const BASE_URL = process.env.WECHAT_EXPORTER_URL || 'http://127.0.0.1:3200';

const llmConfig = await getLLMConfig();
const industryConfig = await getIndustryConfig();

const USE_LLM = !!llmConfig;

// 计算时间范围：支持自定义时间或默认使用昨天7点 - 今天7点
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

  const yesterday7 = new Date(yesterday);
  yesterday7.setHours(7, 0, 0, 0);

  const today7 = new Date(today);
  today7.setHours(7, 0, 0, 0);

  return {
    START_TIME: Math.floor(yesterday7.getTime() / 1000),
    END_TIME: Math.floor(today7.getTime() / 1000)
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

  // [已禁用外部API] 原LLM去重逻辑已注释，改由AI Agent直接分析
  // try {
  //   const newsText = newsList.map((n, i) => {
  //     return `${i + 1}. [${n.industry_type}] ${n.title}\n    来源: ${n.account}\n    内容: ${n.summary.substring(0, 200)}...`;
  //   }).join('\n\n');
  //   const prompt = `分析以下${newsList.length}条新闻，严格识别重复新闻。
// 新闻列表：
// ${newsText}
// 判断标准：
// 1. 同一事件的不同报道 = 重复
// 2. 同一公司同一动作 = 重复
// 3. 同一产品/项目的报道 = 重复
// 4. 相似但不完全相同的不算重复
// 返回JSON格式（只返回JSON）：
// \`\`\`json
// {
//   "duplicate_groups": [
//     {
//       "indices": [1, 3, 5],
//       "keep": 1
//     }
//   ]
// }
// \`\`\`
// 若无重复返回{"duplicate_groups":[]}。`;
  //   const result = await callOpenAI(
  //     prompt,
  //     llmConfig.apiKey,
  //     llmConfig.model,
  //     llmConfig.baseUrl,
  //     llmConfig.provider
  //   );
  //   const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```\s*/) ||
  //                   result.match(/\{[\s\S]*\}/);
  //   if (!jsonMatch) {
  //     return newsList.map(n => ({ ...n, sources: [n.account] }));
  //   }
  //   const jsonStr = jsonMatch[1] || jsonMatch[0];
  //   const parsed = JSON.parse(jsonStr);
  //   if (!parsed.duplicate_groups || !Array.isArray(parsed.duplicate_groups)) {
  //     return newsList.map(n => ({ ...n, sources: [n.account] }));
  //   }
  //   const duplicateMap = new Map();
  //   parsed.duplicate_groups.forEach(group => {
  //     const keepIdx = group.keep - 1;
  //     const indices = group.indices.map(i => i - 1);
  //     indices.forEach(idx => {
  //       if (idx !== keepIdx) {
  //         duplicateMap.set(idx, keepIdx);
  //       }
  //     });
  //     console.log(`    - 识别重复: [${indices.map(x => x + 1).join(', ')}] 归并保留 ${keepIdx + 1}`);
  //   });
  //   const deduplicated = [];
  //   const sourcesMap = new Map();
  //   newsList.forEach((n, i) => {
  //     sourcesMap.set(i, [n.account]);
  //   });
  //   duplicateMap.forEach((targetIdx, sourceIdx) => {
  //     const sourceAccounts = sourcesMap.get(sourceIdx) || [];
  //     const targetAccounts = sourcesMap.get(targetIdx) || [];
  //     const merged = new Set([...sourceAccounts, ...targetAccounts]);
  //     sourcesMap.set(targetIdx, Array.from(merged));
  //   });
  //   const allIndices = new Set([...Array(newsList.length).keys()]);
  //   const duplicateIndices = new Set(duplicateMap.keys());
  //   allIndices.forEach(idx => {
  //     if (!duplicateIndices.has(idx)) {
  //       const news = {
  //         ...newsList[idx],
  //         sources: sourcesMap.get(idx) || [newsList[idx].account]
  //       };
  //       if (news.sources.length > 1) {
  //         news.summary = `${news.summary} [来源: ${news.sources.join(', ')}]`;
  //       }
  //       deduplicated.push(news);
  //     }
  //   });
  //   const removedCount = newsList.length - deduplicated.length;
  //   console.log(`  ✓ 去重完成: 剔除 ${removedCount} 条，保留 ${deduplicated.length} 条`);
  //   return deduplicated;
  // } catch (error) {
  //   console.error(`  ❌ LLM去重失败: ${error.message}`);
  //   return newsList.map(n => ({ ...n, sources: [n.account] }));
  // }
  console.log('  ⚠ 外部LLM API已禁用，去重由AI Agent完成');
  return newsList.map(n => ({ ...n, sources: [n.account] }));
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

    // [已禁用外部API] 原LLM分析逻辑已注释，改由AI Agent直接分析
    // if (USE_LLM && llmConfig) {
    //   console.log('   正在使用LLM分析...');
    //   const analysisConfig = {
    //     ...llmConfig,
    //     industryName: industryConfig.name,
    //     industryKeywords: industryConfig.keywords,
    //     industryCategories: industryConfig.categories
    //   };
    //   const analysisResult = await analyzeArticleWithLLM(article.title, content, analysisConfig);
    //   if (analysisResult.has_news && Array.isArray(analysisResult.news_list)) {
    //     const newsList = analysisResult.news_list.map(news => ({
    //       title: news.title,
    //       summary: news.summary,
    //       industry_type: news.industry_type,
    //       news_type: news.news_type,
    //       link: article.link,
    //       account: article.account,
    //       createTime: article.createTime
    //     }));
    //     console.log(`   ✓ 提取到 ${newsList.length} 条新闻`);
    //     return newsList;
    //   } else {
    //     return [];
    //   }
    // } else {
    //   console.log('   ✗ 未配置LLM');
    //   return [];
    // }
    console.log('   ⚠ 外部LLM API已禁用，请使用 collect-articles.js + AI Agent 方式');
    return [];
  } catch (error) {
    console.error(`   ✗ 处理失败: ${error.message}`);
    return [];
  }
}

/**
 * 使用AI筛选今日主线新闻（默认1条，最多2条）
 */
async function selectTopNews(allNews, llmConfig) {
  if (!llmConfig || allNews.length === 0) {
    return [];
  }

  console.log('\n🔍 AI正在筛选今日重点新闻...');

  const newsSummaries = allNews.map((n, i) =>
    `${i + 1}. ${n.title}\n   ${n.summary.substring(0, 150)}...`
  ).join('\n\n');

  const prompt = `分析以下${allNews.length}条新闻，先提炼当天最适合公众号打开和转发的**一个主线**，再选择能支撑这条主线的新闻。默认只选1条主线，除非当天有互不相关的重大事件，才最多选2条。

判断标准（从多维度综合考量，不是按金额大小）：
1. 打开理由：标题能否写成"变化/冲突/后果"，让非核心读者也想点开？
2. 行业影响：这件事会不会改变数据中心、算力或AI基础设施的资源配置？
3. 主线能力：能否串联2-3条新闻形成一个清晰故事，而不是孤立事件？
4. 趋势信号：反映了什么正在发生的变化，例如拿电、融资、供配电、监管、芯片、客户需求？
5. 读者相关性：数据中心运营商、设备商、投资人、园区/能源侧能不能立刻知道"我该看什么"？

不要只看融资额大小。例如：
- 一家小公司突破某技术瓶颈，可能比大额融资更重要
- 某政策变化，可能影响整个行业
- 某危机事件，可能揭示行业风险

写作要求：
- 先提出候选核心观点，再对每个候选观点逐条网络搜索验证；不得只凭模型记忆直接下结论。
- 必须验证标题前半句、今日判断、今日主线、为什么重要、谁该关注、下一步看什么。
- 每个核心观点至少有1个可靠来源支撑；融资金额、政策、订单、投产、芯片/AI模型、公司战略等高时效内容优先用2个来源交叉确认。
- 判断要克制，避免标题党式夸张；如果只能找到二手消息，语气要降级；无法验证或来源冲突的观点要删除。
- 每条主线必须回答：发生了什么、为什么重要、谁该关注、下一步看什么。
- 弱新闻、展会稿、招聘稿、软文稿、重复转述默认删除；一分钟速览全篇最多6-8条。

新闻列表：
${newsSummaries}

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "top_news": [
    {
      "index": 1,
      "mainline": "当天主线，用一句人话写清变化/冲突/后果",
      "reason": "为什么重要（1-2句话）",
      "insight": "这说明行业正在发生什么变化（1-2句话）",
      "audience": "谁该关注：公司/环节/岗位/角色",
      "watch": "下一步看什么，例如签约、并网、采购、投产、监管审批、财报披露"
    }
  ]
}
\`\`\`
index使用上述列表的实际编号（1到${allNews.length}）。`;

  // [已禁用外部API] 原LLM筛选重点新闻逻辑已注释，改由AI Agent直接分析
  // try {
  //   const result = await callOpenAI(
  //     prompt,
  //     llmConfig.apiKey,
  //     llmConfig.model,
  //     llmConfig.baseUrl,
  //     llmConfig.provider
  //   );
  //   const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```\s*/) ||
  //                     result.match(/\{[\s\S]*\}/);
  //   if (!jsonMatch) {
  //     console.log('   ⚠ 无法解析AI返回结果');
  //     return [];
  //   }
  //   const jsonStr = jsonMatch[1] || jsonMatch[0];
  //   const parsed = JSON.parse(jsonStr);
  //   if (!parsed.top_news || !Array.isArray(parsed.top_news)) {
  //     console.log('   ✓ 未发现特别重要的新闻');
  //     return [];
  //   }
  //   const topNewsList = [];
  //   parsed.top_news.forEach(item => {
  //     const idx = item.index - 1;
  //     if (idx >= 0 && idx < allNews.length) {
  //       topNewsList.push({
  //         news: allNews[idx],
  //         mainline: item.mainline,
  //         reason: item.reason,
  //         insight: item.insight,
  //         audience: item.audience,
  //         watch: item.watch
  //       });
  //     }
  //   });
  //   console.log(`   ✓ 筛选出 ${topNewsList.length} 条重点新闻`);
  //   return topNewsList;
  // } catch (error) {
  //   console.error(`   ❌ 筛选失败: ${error.message}`);
  //   return [];
  // }
  console.log('   ⚠ 外部LLM API已禁用，重点筛选由AI Agent完成');
  return [];
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

  const categoryLines = industryConfig.categories.map(c =>
    `- ${c.name}：${c.description}`
  ).join('\n');

  const prompt = `将以下${allNews.length}条新闻分类到以下类别中：
${categoryLines}

新闻列表：
${newsList}

返回JSON格式（只返回JSON）：
\`\`\`json
{
  "categories": {
${industryConfig.categories.map(c => `    "${c.name}": []`).join(',\n')}
  }
}
\`\`\`
index使用上述列表的实际编号（1到${allNews.length}）。每条新闻只能属于一个类别。`;

  // [已禁用外部API] 原LLM分类逻辑已注释，改由AI Agent直接分析
  // try {
  //   const result = await callOpenAI(
  //     prompt,
  //     llmConfig.apiKey,
  //     llmConfig.model,
  //     llmConfig.baseUrl,
  //     llmConfig.provider
  //   );
  //   const jsonMatch = result.match(/```json\s*([\s\S]*?)\s*```\s*/) ||
  //                     result.match(/\{[\s\S]*\}/);
  //   if (!jsonMatch) {
  //     console.log('   ⚠ 无法解析AI返回结果，归入"其他"');
  //     return { '其他': allNews };
  //   }
  //   const jsonStr = jsonMatch[1] || jsonMatch[0];
  //   const parsed = JSON.parse(jsonStr);
  //   if (!parsed.categories || typeof parsed.categories !== 'object') {
  //     return { '其他': allNews };
  //   }
  //   const categorized = {};
  //   const emojiMap = {};
  //   for (const cat of industryConfig.categories) {
  //     emojiMap[cat.name] = cat.emoji;
  //   }
  //   for (const [category, indices] of Object.entries(parsed.categories)) {
  //     if (Array.isArray(indices) && indices.length > 0) {
  //       const key = emojiMap[category] ? category : '其他';
  //       categorized[key] = indices.map(idx => allNews[idx - 1]).filter(n => n);
  //     }
  //   }
  //   const categorizedNews = new Set();
  //   Object.values(categorized).forEach(list => list.forEach(n => categorizedNews.add(n)));
  //   if (categorizedNews.size < allNews.length) {
  //     const uncategorized = allNews.filter(n => !categorizedNews.has(n));
  //     if (!categorized['其他']) categorized['其他'] = [];
  //     categorized['其他'].push(...uncategorized);
  //   }
  //   const totalCount = Object.values(categorized).reduce((sum, list) => sum + list.length, 0);
  //   console.log(`   ✓ 分类完成: ${totalCount} 条`);
  //   return categorized;
  // } catch (error) {
  //   console.error(`   ❌ 分类失败: ${error.message}`);
  //   return { '其他': allNews };
  // }
  console.log('   ⚠ 外部LLM API已禁用，分类由AI Agent完成');
  return { '其他': allNews };
}

/**
 * 生成Markdown报告（公众号增长版）
 */
async function generateMarkdownReport(startDate, endDate, allNews, topNewsList, categorizedNews) {
  const today = new Date();
  const dateStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
  const shortDate = `${today.getMonth() + 1}.${today.getDate()}`;

  const titleLead = topNewsList[0]?.mainline || topNewsList[0]?.insight || topNewsList[0]?.news.title || '今天算力产业最该看的变化';
  const shortTitleLead = titleLead.length > 22 ? titleLead.substring(0, 22) : titleLead;
  let report = `# ${shortTitleLead}｜${shortDate}数据中心、算力、AI日报\n\n`;

  const judgmentLead = topNewsList.length > 0
    ? `${topNewsList[0].mainline || topNewsList[0].insight || topNewsList[0].reason || topNewsList[0].news.title}`
    : `今天该时间段缺少足够强的新动态，重点看后续是否出现订单、投产、监管或财报披露。`;

  report += `## 今日判断\n\n`;
  report += `**今天最重要的一句话**：${judgmentLead}\n\n`;
  report += `**三个变化信号**：\n\n`;
  if (topNewsList.length > 0) {
    topNewsList.slice(0, 3).forEach(item => {
      report += `- ${item.reason || item.news.title}\n`;
    });
  } else {
    report += `- 该时间段没有足够强的新主线，宁可少写也不强行凑数。\n`;
  }
  report += `\n`;

  report += `## 今天对谁有用\n\n`;
  const audiences = topNewsList
    .map(item => item.audience || item.beneficiary)
    .filter(Boolean);
  if (audiences.length > 0) {
    audiences.slice(0, 4).forEach(audience => {
      report += `- ${audience}\n`;
    });
  } else {
    report += `- 数据中心运营商：重点看电力、客户和交付节奏。\n`;
    report += `- 设备与工程厂商：重点看供配电、液冷、储能和网络侧新增需求。\n`;
    report += `- 投资人与园区侧：重点看融资、政策、并网和长期订单。\n`;
  }
  report += `\n`;

  report += `---\n\n`;

  if (topNewsList.length > 0) {
    report += `## 🔥 今日主线\n\n`;
    topNewsList.slice(0, 2).forEach((item, idx) => {
      const news = item.news;
      report += `### ${idx + 1}. ${news.title}\n\n`;
      if (news.link) {
        report += `[原文链接](${news.link})\n\n`;
      }
      report += `**发生了什么**：${news.summary}\n\n`;
      report += `**为什么重要**：${item.reason}\n\n`;
      report += `**行业变化**：${item.insight || '待验证'}\n\n`;
      report += `**谁该关注**：${item.audience || item.beneficiary || '相关数据中心、算力、AI基础设施从业者'}\n\n`;
      report += `**下一步看什么**：${item.watch || '关注后续订单、采购、投产、并网或监管披露'}\n\n`;
      report += `---\n\n`;
    });
  }

  report += `## ⚡ 一分钟速览\n\n`;

  const categories = industryConfig.categories.map(c => ({
    key: `${c.emoji} ${c.name}`,
    name: c.name
  }));

  const topNewsTitles = new Set(topNewsList.map(n => n.news.title));

  let speedCount = 0;
  const maxSpeedItems = 8;

  for (const cat of categories) {
    if (speedCount >= maxSpeedItems) break;
    const news = categorizedNews[cat.name];
    if (news && news.length > 0) {
      const rows = [];

      for (const item of news) {
        if (speedCount >= maxSpeedItems) break;
        if (topNewsTitles.has(item.title)) {
          continue;
        }
        const summary = item.title.length > 30 ? item.title.substring(0, 30) + '...' : item.title;
        const stars = item.impact_stars || '⭐⭐⭐';
        const link = item.link ? `[链接](${item.link})` : '-';
        rows.push(`| ${summary} | ${stars} | ${link} |`);
        speedCount++;
      }

      if (rows.length > 0) {
        report += `### ${cat.key}\n\n`;
        report += `| 动态概要 | 影响度 | 原文链接 |\n`;
        report += `|---------|-------|---------|\n`;
        report += rows.join('\n') + '\n';
        report += `\n`;
      }
    }
  }

  report += `> **关注我们**：持续追踪算力、数据中心与AI基础设施的资金、电力、技术和政策信号，帮你从密集信息里抓住真正值得行动的产业变化。\n\n`;

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
  const outputDir = 'd:\\Amateurinterests\\微信公众号分析\\output';
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  const now = new Date();
  const dateStr = `${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`;
  const mdPath = `${outputDir}\\${industryConfig.name}动态日报${dateStr}.md`;
  fs.writeFileSync(mdPath, report, 'utf-8');

  console.log(`\n报告已保存: ${mdPath}`);
  console.log('\n=== 完成 ===');
}

main().catch(error => {
  console.error('程序出错:', error);
  process.exit(1);
});
