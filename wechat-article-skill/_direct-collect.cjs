const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const exporterDir = path.join(root, 'wechat-article-exporter');
const outputDir = path.join(root, 'output');
const authKey = process.env.AUTH_KEY || readEnvAuthKey();

function readEnvAuthKey() {
  const envPath = path.join(__dirname, '.env');
  const env = fs.readFileSync(envPath, 'utf8');
  const match = env.match(/^AUTH_KEY=(.+)$/m);
  if (!match) throw new Error('AUTH_KEY not found in .env');
  return match[1].trim().replace(/^['"]|['"]$/g, '');
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function getAccounts() {
  const accountDir = path.join(exporterDir, '.data', 'kv', 'account');
  return fs.readdirSync(accountDir)
    .map(name => readJson(path.join(accountDir, name)))
    .sort((a, b) => (b.create_time || 0) - (a.create_time || 0));
}

function getCookieData() {
  const cookiePath = path.join(exporterDir, '.data', 'kv', 'cookie', authKey);
  const data = readJson(cookiePath);
  const cookie = data.cookies
    .filter(c => c.value && c.value !== 'EXPIRED')
    .map(c => `${c.name}=${c.value}`)
    .join('; ');
  return { token: data.token, cookie };
}

async function fetchWithTimeout(url, options = {}, timeoutMs = 20000) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(url, { ...options, signal: controller.signal });
  } finally {
    clearTimeout(timer);
  }
}

async function getArticles(account, session, size = 200) {
  const query = new URLSearchParams({
    sub: 'list',
    search_field: 'null',
    begin: '0',
    count: String(size),
    query: '',
    fakeid: account.fakeid,
    type: '101_1',
    free_publish_type: '1',
    sub_action: 'list_ex',
    token: session.token,
    lang: 'zh_CN',
    f: 'json',
    ajax: '1',
  });
  const response = await fetchWithTimeout(`https://mp.weixin.qq.com/cgi-bin/appmsgpublish?${query}`, {
    headers: {
      Referer: 'https://mp.weixin.qq.com/',
      Origin: 'https://mp.weixin.qq.com',
      'User-Agent': 'Mozilla/5.0',
      'Accept-Encoding': 'identity',
      Cookie: session.cookie,
    },
  });
  const data = await response.json();
  if (data.base_resp?.ret !== 0) return [];

  const publishPage = JSON.parse(data.publish_page);
  return publishPage.publish_list.flatMap(item => {
    const publishInfo = JSON.parse(item.publish_info);
    const publishTime = publishInfo.sent_info?.time;
    const messages = publishInfo.appmsgex || publishInfo.appmsg_info || [];
    return messages.map(appmsg => ({
      ...appmsg,
      publish_time: publishTime || appmsg.create_time,
    }));
  });
}

function htmlToText(html) {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/<[^>]+>/g, '\n')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\n{3,}/g, '\n\n')
    .replace(/[ \t]{2,}/g, ' ')
    .trim();
}

async function getArticleContent(link, session) {
  try {
    const response = await fetchWithTimeout(link, {
      headers: {
        'User-Agent': 'Mozilla/5.0',
        'Accept-Encoding': 'identity',
        Cookie: session.cookie,
      },
    }, 25000);
    if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
    return htmlToText(await response.text());
  } catch (error) {
    console.error(`  warning: cannot fetch content - ${error.message}`);
    return null;
  }
}

async function main() {
  const [startArg, endArg] = process.argv.slice(2);
  if (!startArg || !endArg) {
    console.error('Usage: node _direct-collect.cjs "YYYY-MM-DD HH:mm" "YYYY-MM-DD HH:mm"');
    process.exit(1);
  }
  const start = new Date(startArg);
  const end = new Date(endArg);
  const startTs = Math.floor(start.getTime() / 1000);
  const endTs = Math.floor(end.getTime() / 1000);
  const session = getCookieData();
  const accounts = getAccounts();

  console.log(`Direct collect: ${startArg} -> ${endArg}`);
  console.log(`Accounts: ${accounts.length}`);

  const allArticles = [];
  for (const account of accounts) {
    const articles = await getArticles(account, session);
    const recent = articles.filter(a => a.publish_time >= startTs && a.publish_time <= endTs);
    console.log(`  ${account.nickname}: ${recent.length}`);
    for (const article of recent) {
      allArticles.push({
        title: article.title,
        link: article.link,
        account: account.nickname,
        createTime: article.publish_time,
      });
    }
  }

  console.log(`Found ${allArticles.length} articles`);
  const collected = [];
  for (let i = 0; i < allArticles.length; i++) {
    const article = allArticles[i];
    console.log(`[${i + 1}/${allArticles.length}] ${article.title.slice(0, 70)}`);
    const content = await getArticleContent(article.link, session);
    if (content) {
      collected.push({ ...article, content: content.slice(0, 15000) });
    }
    if (i < allArticles.length - 1) await new Promise(resolve => setTimeout(resolve, 1000));
  }

  const mmdd = `${String(new Date().getMonth() + 1).padStart(2, '0')}${String(new Date().getDate()).padStart(2, '0')}`;
  const out = path.join(outputDir, `collected-articles-${mmdd}.json`);
  fs.writeFileSync(out, JSON.stringify({
    industryName: '算力，数据中心，AI',
    industryKeywords: ['算力', '数据中心', 'AI', '人工智能', 'GPU', '芯片', '智算'],
    industryCategories: ['融资', '建设', '技术', '风险', '其他'],
    timeRange: { start: startArg, end: endArg },
    articleCount: collected.length,
    articles: collected,
  }, null, 2), 'utf8');
  console.log(`Saved ${out}`);
}

main().catch(error => {
  console.error(error);
  process.exit(1);
});
