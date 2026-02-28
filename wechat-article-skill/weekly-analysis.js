#!/usr/bin/env node

/**
 * 获取并分析最近一周的新闻 - 逐条处理版本
 */

const { request } = require('./auth-manager.js');

const BASE_URL = 'http://localhost:3200';

// 计算时间范围：最近 7 天
const DAYS = 7;
const endTime = new Date();
endTime.setHours(18, 0, 0, 0);
const END_TIME = Math.floor(endTime.getTime() / 1000);
const START_TIME = END_TIME - (DAYS * 24 * 60 * 60);

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
    return publishInfo.appmsgex;
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
 * 拆分文章中的多条新闻
 */
function splitNewsItems(title, content) {
  const items = [];

  // 清理内容
  let cleanContent = content || '';

  // 移除无关内容
  cleanContent = cleanContent
    .replace(/在小说阅读器中沉浸阅读[\s\S]*?global\s+wmedia\s+global/gi, '')
    .replace(/原创\s*[A-Z][a-z]+(?:\s+[A-Z][a-z]+)?/g, '')
    .replace(/global\s+wmedia\s+global/gi, '')
    .replace(/点击关注.*资讯/g, ' ')
    .replace(/阅读原文/g, ' ')
    .replace(/左右滑动/g, ' ')
    .replace(/-?\s*End/g, ' ')
    .replace(/相关新闻资讯[\s\S]*/gi, '')
    .replace(/我们的赞助商伙伴[\s\S]*/gi, '')
    .replace(/参会和赞助咨询[\s\S]*/gi, '')
    .replace(/联系方式.*详情/gi, '')
    .replace(/图：[\s\S]*?图片来源/gi, '')
    .replace(/图片来源[\s\S]*/gi, '')
    .replace(/[\u2000-\u200F\u2028-\u202F\u205F-\u206F\uFEFF\u3000]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();

  // 检查标题是否有 "|" 分隔符
  if (title.includes(' | ')) {
    const parts = title.split(' | ');
    // 第一个部分通常是栏目名，后续是新闻标题
    for (let i = 1; i < parts.length; i++) {
      if (parts[i].trim()) {
        items.push({
          title: parts[i].trim(),
          content: cleanContent,
          sourceTitle: title
        });
      }
    }
    return items;
  }

  // 检查是否是每日大事件类型文章（包含多个新闻摘要）
  if (title.includes('每日大事件') || cleanContent.includes('☆')) {
    // 按 ☆ 符号拆分
    const newsList = cleanContent.split('☆').filter(item => item.trim().length > 10);
    if (newsList.length > 1) {
      for (const news of newsList) {
        const newsText = news.trim();
        if (newsText.length > 10) {
          items.push({
            title: newsText.substring(0, 50),
            content: newsText,
            sourceTitle: title
          });
        }
      }
      if (items.length > 0) {
        return items;
      }
    }
  }

  // 尝试按数字编号拆分 (1. 2. 3. 或 一、二、三、)
  const numberedNewsPatterns = [
    /(?:^|\n)([\d一二三四五六七八九十]+[、.。]\s*[^。\n]+)/g,
    /(?:^|\n)(\d+[、.。]\s*[^。\n]+)/g,
    /(?:^|\n)([-•\*]\s*[^。\n]+)/g,
  ];

  for (const pattern of numberedNewsPatterns) {
    const matches = [...cleanContent.matchAll(pattern)];
    if (matches.length > 2) { // 如果找到至少2条编号新闻
      for (const match of matches) {
        let newsText = match[1] || match[0];
        newsText = newsText.replace(/^[\d一二三四五六七八九十+\-•\*、.。]\s*/, '').trim();
        if (newsText.length > 10) {
          items.push({
            title: newsText.substring(0, 50),
            content: newsText,
            sourceTitle: title
          });
        }
      }
      if (items.length > 0) {
        return items;
      }
    }
  }

  // 如果没有拆分，整篇文章作为一条
  items.push({
    title: title,
    content: cleanContent,
    sourceTitle: title
  });

  return items;
}

function analyzeArticle(title, content, link) {
  const text = (content || '') + ' ' + title;

  // 剔除非新闻类内容
  const excludeKeywords = [
    '招聘', '热招', '人才', '岗位', '诚聘',
    '课程', '学习', '教程', '培训',
    '推荐', '重磅', '福利',
    '展位预订', '赞助招商',
  ];

  for (const keyword of excludeKeywords) {
    if (title.includes(keyword)) {
      return { type: 'exclude', reason: `包含关键词: ${keyword}` };
    }
  }

  // 行业类型识别
  const industryTypes = {
    '数据中心': ['数据中心', 'IDC', '机房', '服务器', '零碳算力', '数字基础设施', 'Starlink', '卫星'],
    '算力': ['算力', 'AI芯片', 'GPU', '英伟达', 'ARM', '芯片设计', '零碳算力'],
    '云计算': ['云计算', '云服务', '公有云', '私有云', '混合云'],
    '人工智能': ['人工智能', 'AI', '深度学习', 'LLM', '大模型', 'DeepSeek', '华为手机', 'Engram',
                 'OceanBit', '温差能', '海洋'],
    '大数据': ['大数据', '数据中台', '数据治理'],
    '跨境数据': ['跨境', '数据出境', '国际数据', '印尼', '泰国', '新加坡'],
  };

  const matchedIndustries = [];
  for (const [industry, keywords] of Object.entries(industryTypes)) {
    for (const keyword of keywords) {
      if (text.includes(keyword)) {
        if (!matchedIndustries.includes(industry)) {
          matchedIndustries.push(industry);
        }
        break;
      }
    }
  }

  // 必须是目标行业之一
  if (matchedIndustries.length === 0) {
    return { type: 'exclude', reason: '不属于目标行业类型' };
  }

  // 新闻类型识别
  const newsTypes = {
    '融资投资': ['融资', '投资', '资金', '入股', '重仓', '上市', 'IPO', '亿美元', '亿'],
    '政策法规': ['政策', '法规', '批准', '监管', '合规', 'FCC'],
    '市场动态': ['市场', '发布', '推出', '增长', '份额', '重返', '中国', '市场第一'],
    '技术创新': ['技术', '论文', '专利', '研发', '创新', '开源', '蓝图'],
    '财务报告': ['财报', '营收', '利润', '业绩'],
    '战略合作': ['合作', '签署', '协议', '联盟', '伙伴'],
    '会展信息': ['大会', '展会', '论坛', '峰会', '举办', '无锡'],
    '项目动态': ['项目', '计划', '建设', '扩张', '落地', '园区', '设施'],
  };

  const matchedNewsTypes = [];
  for (const [newsType, keywords] of Object.entries(newsTypes)) {
    for (const keyword of keywords) {
      if (text.includes(keyword)) {
        if (!matchedNewsTypes.includes(newsType)) {
          matchedNewsTypes.push(newsType);
        }
        break;
      }
    }
  }

  // 必须是新闻类型之一
  if (matchedNewsTypes.length === 0) {
    return { type: 'exclude', reason: '不属于目标新闻类型' };
  }

  // 提取摘要
  let summary = '';

  if (content && content.length > 0) {
    const sentences = content.split(/[。！？;；\n]/).filter(s => s.trim().length > 15);

    const keyPatterns = [
      /[\d一二三四五六七八九十]+.*[万亿]?美元?/m,
      /[0-9]+[Mm][Ww]/m,
      /[0-9]+\s*[万千亿]?[元美]/m,
      /签署|批准|发布|推出|投资|合作|达成|m/gm,
      /计划|项目|建设|扩张|协议|园区|部署/gm,
      /公司|集团|云科技|数据中心|mW|GW|AWS|ARM|OpenAI|Cerebras|OceanBit|SpaceX|FCC|BOI/gim,
    ];

    const keySentences = sentences.filter(s => keyPatterns.some(p => p.test(s)));

    const maxLen = 500;

    if (keySentences.length > 0) {
      summary = keySentences.slice(0, 3).join('。');
    } else {
      summary = content.substring(0, maxLen);
    }

    if (summary.length > maxLen) {
      summary = summary.substring(0, maxLen).trim();
      const lastPeriod = Math.max(
        summary.lastIndexOf('。'),
        summary.lastIndexOf('！'),
        summary.lastIndexOf('？')
      );
      if (lastPeriod > maxLen * 0.5) {
        summary = summary.substring(0, lastPeriod + 1);
      } else {
        summary += '...';
      }
    } else if (summary.length > 0 && !summary.endsWith('。') && !summary.endsWith('！') && !summary.endsWith('？')) {
      summary += '。';
    }
  } else {
    summary = title;
  }

  return {
    type: 'news',
    industries: matchedIndustries,
    newsTypes: matchedNewsTypes,
    summary: summary || title
  };
}

/**
 * 处理单篇文章，返回分析后的新闻列表
 */
async function processSingleArticle(article, index, total) {
  console.log(`\n[${index}/${total}] ${article.title.substring(0, 60)}...`);
  console.log(`   链接: ${article.link}`);

  const results = [];

  try {
    // 获取文章内容
    console.log('   正在获取内容...');
    const content = await getArticleContent(article.link);

    if (!content) {
      console.log('   ✗ 无法获取文章内容，跳过');
      return { news: [], excluded: [] };
    }

    // 拆分新闻条目
    console.log('   正在拆分新闻条目...');
    const newsItems = splitNewsItems(article.title, content);
    console.log(`   → 拆分出 ${newsItems.length} 条新闻`);

    // 分析每条新闻
    for (let i = 0; i < newsItems.length; i++) {
      const item = newsItems[i];
      const analysis = analyzeArticle(item.title, item.content, article.link);

      if (analysis.type === 'news') {
        results.push({
          title: item.title,
          link: article.link,
          account: article.account,
          createTime: article.createTime,
          sourceTitle: item.sourceTitle,
          analysis
        });
        console.log(`   ✓ [${i + 1}] ${analysis.industries.join(', ')} | ${analysis.newsTypes.join(', ')}`);
      } else {
        results.push({
          title: item.title,
          reason: analysis.reason,
          sourceTitle: item.sourceTitle,
          excluded: true
        });
        console.log(`   ✗ [${i + 1}] ${analysis.reason}`);
      }
    }
  } catch (error) {
    console.error(`   ✗ 处理失败: ${error.message}`);
  }

  return {
    news: results.filter(r => !r.excluded),
    excluded: results.filter(r => r.excluded)
  };
}

async function main() {
  const startDate = new Date(START_TIME * 1000).toLocaleDateString('zh-CN');
  const endDate = new Date(END_TIME * 1000).toLocaleDateString('zh-CN');

  console.log('=== 新闻分析报告（逐条处理）===');
  console.log(`时间范围: 最近 ${DAYS} 天 (${startDate} 至 ${endDate})\n`);

  const accounts = await getAccounts();
  console.log(`共有 ${accounts.length} 个公众号\n`);

  // 收集所有文章
  console.log('正在收集文章列表...\n');
  let allArticles = [];

  for (const account of accounts) {
    const articles = await getArticles(account.fakeid);
    const recentArticles = articles.filter(a => a.create_time >= START_TIME);
    console.log(`${account.nickname}: ${recentArticles.length} 篇`);

    for (const article of recentArticles) {
      allArticles.push({
        title: article.title,
        link: article.link,
        account: account.nickname,
        createTime: article.create_time
      });
    }
  }

  console.log(`\n共找到 ${allArticles.length} 篇文章\n`);
  console.log('开始逐条分析...\n');
  console.log('='.repeat(60));

  // 逐条处理文章
  const newsArticles = [];
  const excludedArticles = [];
  let successCount = 0;
  let failCount = 0;

  for (let i = 0; i < allArticles.length; i++) {
    const result = await processSingleArticle(allArticles[i], i + 1, allArticles.length);

    newsArticles.push(...result.news);
    excludedArticles.push(...result.excluded);

    if (result.news.length + result.excluded.length > 0) {
      successCount++;
    } else {
      failCount++;
    }

    // 避免请求过快
    if (i < allArticles.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 500));
    }
  }

  console.log('\n' + '='.repeat(60));
  console.log(`\n=== 分析完成 ===`);
  console.log(`处理文章: ${successCount} 成功, ${failCount} 失败`);
  console.log(`新闻条目: ${newsArticles.length + excludedArticles.length} 条`);
  console.log(`收录新闻: ${newsArticles.length} 条`);
  console.log(`剔除新闻: ${excludedArticles.length} 条\n`);

  // 统计
  const industryCount = {};
  newsArticles.forEach(a => {
    a.analysis.industries.forEach(i => {
      industryCount[i] = (industryCount[i] || 0) + 1;
    });
  });

  const newsTypeCount = {};
  newsArticles.forEach(a => {
    a.analysis.newsTypes.forEach(t => {
      newsTypeCount[t] = (newsTypeCount[t] || 0) + 1;
    });
  });

  // 生成公众号格式的报告
  const wechatReport = generateWechatReport(startDate, endDate, allArticles, successCount, newsArticles, excludedArticles, industryCount, newsTypeCount, newsArticles.map(a => a.account).filter((v, i, a) => a.indexOf(v) === i).sort().join('、'));

  // 生成 Markdown 格式的报告（存档用）
  const mdReport = generateMarkdownReport(startDate, endDate, allArticles, successCount, newsArticles, excludedArticles, industryCount, newsTypeCount);

  const fs = await import('fs');
  const outputDir = 'd:\\业余兴趣\\新闻分析结果';
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);

  // 保存公众号格式（HTML）
  const wechatReportPath = `${outputDir}\\wechat-${timestamp}.html`;
  fs.writeFileSync(wechatReportPath, wechatReport, 'utf-8');
  console.log(`\n公众号格式已保存: ${wechatReportPath}`);
  console.log(`(请用浏览器打开文件，复制内容到微信编辑器)`);

  // 保存 Markdown 格式（存档）
  const mdReportPath = `${outputDir}\\weekly-news-analysis-${timestamp}.md`;
  fs.writeFileSync(mdReportPath, mdReport, 'utf-8');
  console.log(`Markdown 存档已保存: ${mdReportPath}\n`);

  // 输出公众号格式到控制台
  console.log('='.repeat(60));
  console.log('【公众号排版格式】可直接复制到微信编辑器');
  console.log('='.repeat(60));
  console.log(wechatReport);
}

/**
 * 生成适合公众号排版的格式（HTML富文本）
 */
function generateWechatReport(startDate, endDate, allArticles, successCount, newsArticles, excludedArticles, industryCount, newsTypeCount, accountsStr) {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  const dateStr = `${month}${day}`;

  let report = '';

  // 版式装饰线
  report += `<section style="box-sizing: border-box; font-size: 16px; font-family: -apple-system, BlinkMacSystemFont, 'Helvetica Neue', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', SimHei, Arial, sans-serif; letter-spacing: 0.544px; text-align: center; font-weight: bold; color: rgb(62, 62, 62);">`;
  report += `<section style="padding: 10px; margin-bottom: 10px; text-align: center; color: rgb(69, 173, 148); font-size: 18px; font-weight: bold;">`;
  report += `算力、数据中心、AI动态周报${dateStr}`;
  report += `</section>`;

  // 统计信息
  report += `<section style="margin: 20px 0; padding: 15px; background-color: rgb(249, 249, 249); border-left: 4px solid rgb(69, 173, 148);">`;
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);"><strong>📊 本周数据统计</strong></p>`;
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);">收录新闻：<strong style="color: rgb(69, 173, 148);">${newsArticles.length}</strong> 条</p>`;
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);">剔除新闻：<strong style="color: rgb(230, 78, 78);">${excludedArticles.length}</strong> 条</p>`;
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);">数据来源：${accountsStr}</p>`;
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);">统计时间：${startDate} 至 ${endDate}</p>`;
  report += `</section>`;

  // 行业分布
  report += `<section style="margin: 20px 0;">`;
  report += `<p style="margin: 10px 0; color: rgb(69, 173, 148); font-size: 16px; font-weight: bold;"><strong>🏢 行业分布</strong></p>`;
  const topIndustries = Object.entries(industryCount)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 3);
  topIndustries.forEach(([ind, count], i) => {
    const colors = ['rgb(255, 163, 0)', 'rgb(192, 192, 192)', 'rgb(205, 127, 50)'];
    report += `<p style="margin: 5px 0; color: ${colors[i]};"><strong>${ind}</strong>：${count} 条</p>`;
  });
  report += `</section>`;

  // 新闻类型分布
  report += `<section style="margin: 20px 0;">`;
  report += `<p style="margin: 10px 0; color: rgb(69, 173, 148); font-size: 16px; font-weight: bold;"><strong>📰 新闻类型</strong></p>`;
  const topTypes = Object.entries(newsTypeCount)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5);
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);">`;
  report += topTypes.map(([type, count]) => `<strong>${type}</strong> ${count} 条`).join(' | ');
  report += `</p>`;
  report += `</section>`;

  // 分隔线
  report += `<section style="margin: 20px 0; text-align: center; color: rgb(217, 217, 217);">`;
  report += `─────────────────────`;
  report += `</section>`;

  // 新闻详情
  report += `<section style="margin: 20px 0;">`;
  report += `<p style="margin: 10px 0; color: rgb(69, 173, 148); font-size: 18px; font-weight: bold; text-align: center;"><strong>📋 本周新闻详情</strong></p>`;
  report += `</section>`;

  newsArticles.forEach((article, i) => {
    report += `<section style="margin: 15px 0; padding: 0; border: 1px solid rgb(230, 230, 230); border-left: 3px solid rgb(69, 173, 148); border-radius: 6px; background-color: rgb(255, 255, 255); overflow: hidden;">`;
    report += `<p style="margin: 0; padding: 12px; color: rgb(51, 51, 51); font-size: 16px; line-height: 1.6;">`;
    report += `<a href="${article.link}" style="color: rgb(51, 51, 51); text-decoration: none; font-weight: 500;">${article.title}</a>`;
    report += `</p>`;
    report += `<p style="margin: 0; padding: 0 12px 12px; color: rgb(102, 102, 102); font-size: 15px; line-height: 1.8;">`;
    report += article.analysis.summary;
    report += `</p>`;
    report += `</section>`;
  });

  // 底部
  report += `<section style="margin: 30px 0; padding: 15px; background-color: rgb(242, 242, 242); border-radius: 8px; text-align: center;">`;
  report += `<p style="margin: 5px 0; color: rgb(62, 62, 62);"><strong style="color: rgb(69, 173, 148);">💡 本周热点总结</strong></p>`;
  report += `<p style="margin: 10px 0; color: rgb(62, 62, 62); line-height: 1.8;">`;
  report += `本周共收录 <strong style="color: rgb(69, 173, 148);">${newsArticles.length}</strong> 条行业动态，涵盖 <strong style="color: rgb(69, 173, 148);">${Object.keys(industryCount).length}</strong> 个行业领域。<br/>`;
  report += `最受关注的是「<strong style="color: rgb(69, 173, 148);">${topIndustries[0]?.[0] || '数据中心'}</strong>」相关新闻（${topIndustries[0]?.[1] || 0} 条）。`;
  report += `</p>`;
  report += `<p style="margin: 15px 0 5px 0; color: rgb(128, 128, 128); font-size: 14px;">欢迎持续关注本栏目！</p>`;
  report += `</section>`;

  report += `</section>`;

  return report;
}

/**
 * 生成 Markdown 格式的报告（存档用）
 */
function generateMarkdownReport(startDate, endDate, allArticles, successCount, newsArticles, excludedArticles, industryCount, newsTypeCount) {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  const dateStr = `${month}${day}`;

  let report = '';
  report += `# （算力、数据中心、AI动态周报${dateStr}）\n\n`;
  report += `## 时间范围\n最近 ${DAYS} 天 (${startDate} 至 ${endDate})\n\n`;

  report += '## 统计\n\n';
  report += `- 原始文章: ${allArticles.length} 篇\n`;
  report += `- 处理文章: ${successCount} 篇成功\n`;
  report += `- 新闻条目: ${newsArticles.length + excludedArticles.length} 条\n`;
  report += `- 收录新闻: ${newsArticles.length} 条\n`;
  report += `- 剔除新闻: ${excludedArticles.length} 条\n\n`;

  report += '### 行业分布\n\n';
  Object.entries(industryCount)
    .sort((a, b) => b[1] - a[1])
    .forEach(([ind, count]) => {
      report += `- ${ind}: ${count} 条\n`;
    });

  report += '\n### 新闻类型分布\n\n';
  Object.entries(newsTypeCount)
    .sort((a, b) => b[1] - a[1])
    .forEach(([type, count]) => {
      report += `- ${type}: ${count} 条\n`;
    });

  report += '\n## 新闻详情\n\n';

  newsArticles.forEach((article, i) => {
    report += `${i + 1}. [${article.title}](${article.link})\n`;
    report += `   ${article.analysis.summary}\n\n`;
  });

  if (excludedArticles.length > 0) {
    report += '## 剔除新闻\n\n';
    excludedArticles.forEach((article, i) => {
      report += `${i + 1}. ${article.title}\n`;
      report += `   原因: ${article.reason}\n`;
    });
    report += '\n';
  }

  return report;
}

main().catch(error => {
  console.error('程序出错:', error);
  process.exit(1);
});
