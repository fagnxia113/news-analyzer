#!/usr/bin/env node

import { request } from './auth-manager.js';
import { getIndustryConfig } from './config-manager.js';

const BASE_URL = process.env.WECHAT_EXPORTER_URL || 'http://127.0.0.1:3200';
const industryConfig = await getIndustryConfig();

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

  console.log('=== 文章收集（无LLM模式）===');
  console.log(`时间范围: ${formatDate(startDate)} 至 ${formatDate(endDate)}`);

  const accounts = await getAccounts();
  console.log(`共有 ${accounts.length} 个公众号\n`);

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

  console.log(`\n✓ 共找到 ${allArticles.length} 篇文章`);

  const collectedArticles = [];

  for (let i = 0; i < allArticles.length; i++) {
    const article = allArticles[i];
    console.log(`\n[${i + 1}/${allArticles.length}] ${article.title.substring(0, 60)}...`);

    const content = await getArticleContent(article.link);

    if (content) {
      collectedArticles.push({
        title: article.title,
        link: article.link,
        account: article.account,
        createTime: article.createTime,
        content: content.substring(0, 15000)
      });
      console.log('   ✓ 已获取内容');
    } else {
      console.log('   ✗ 无法获取内容，跳过');
    }

    if (i < allArticles.length - 1) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }

  const fs = await import('fs');
  const outputDir = 'd:\\Amateurinterests\\微信公众号分析\\output';
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const now = new Date();
  const dateStr = `${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`;
  const jsonPath = `${outputDir}\\collected-articles-${dateStr}.json`;

  fs.writeFileSync(jsonPath, JSON.stringify({
    industryName: industryConfig.name,
    industryKeywords: industryConfig.keywords,
    industryCategories: industryConfig.categories,
    timeRange: { start: formatDate(startDate), end: formatDate(endDate) },
    articleCount: collectedArticles.length,
    articles: collectedArticles
  }, null, 2), 'utf-8');

  console.log(`\n文章数据已保存: ${jsonPath}`);
  console.log(`共收集 ${collectedArticles.length} 篇文章`);
  console.log('\n请使用 AI Agent 读取此文件进行新闻分析');
}

main().catch(error => {
  console.error('程序出错:', error);
  process.exit(1);
});
