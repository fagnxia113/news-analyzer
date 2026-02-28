#!/usr/bin/env node

/**
 * 获取所有公众号最近指定天数内发布的文章
 */

const { request } = require('./auth-manager.js');

const BASE_URL = 'http://localhost:3200';

// 计算时间范围：最近 7 天
const DAYS = 7;
const END_TIME = Math.floor(Date.now() / 1000); // 当前时间
const START_TIME = END_TIME - (DAYS * 24 * 60 * 60); // 7天前

async function getAccounts() {
  const response = await request(`${BASE_URL}/api/web/mp/accounts`);
  return response.list;
}

async function getArticles(fakeid, size = 50) {
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

async function main() {
  const startDate = new Date(START_TIME * 1000).toLocaleDateString('zh-CN');
  const endDate = new Date(END_TIME * 1000).toLocaleDateString('zh-CN');
  console.log(`=== 获取最近 ${DAYS} 天发布的文章 ===\n`);
  console.log(`时间范围: ${startDate} 至 ${endDate}\n`);

  const accounts = await getAccounts();
  console.log(`共有 ${accounts.length} 个公众号\n`);

  let totalArticles = 0;
  const allRecentArticles = [];

  for (const account of accounts) {
    const articles = await getArticles(account.fakeid);

    // 过滤出指定时间内发布的文章
    const recentArticles = articles.filter(a => a.create_time >= START_TIME);

    if (recentArticles.length > 0) {
      console.log(`\n${account.nickname} (${account.fakeid})`);
      console.log(`  最近发布 ${recentArticles.length} 篇文章:\n`);

      for (const article of recentArticles) {
        const date = new Date(article.create_time * 1000).toLocaleString('zh-CN');
        console.log(`    - ${article.title}`);
        console.log(`      时间: ${date}`);
        console.log(`      链接: ${article.link}`);
        console.log('');
        allRecentArticles.push({
          title: article.title,
          link: article.link,
          account: account.nickname,
          createTime: article.create_time,
          date: date
        });
      }

      totalArticles += recentArticles.length;
    }
  }

  if (totalArticles === 0) {
    console.log('没有找到符合条件的文章。');
  } else {
    console.log(`\n=== 总计 ${totalArticles} 篇文章 ===`);
    console.log('\n=== JSON 格式输出 ===');
    console.log(JSON.stringify(allRecentArticles, null, 2));
  }
}

main().catch(console.error);
