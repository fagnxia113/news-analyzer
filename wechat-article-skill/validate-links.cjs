#!/usr/bin/env node
/**
 * 微信链接预验证与过滤
 *
 * 问题背景：微信编辑器在保存草稿时，会对正文里的所有 <a href="..."> 链接做服务端验证
 * 验证失败的链接会触发"请勿输入不合法的消息图文链接"错误，导致整篇草稿保存失败
 *
 * 常见触发原因（按概率排序）：
 * 1. 同一篇文章被多篇日报引用（如"每日大事件"汇总类文章），使用频次过高
 * 2. 链接被微信风控标记（RetKey: 14），触发了 appmsgcaptcha 验证码
 * 3. 文章已删除、私密、被封禁
 *
 * 本脚本的策略：
 * - 用 curl -I 模拟微信编辑器的验证请求（带真实 UA + Referer）
 * - 检查响应头中的 RetKey（14 = 风控验证码，11/13 = 正常）
 * - 标记"风险链接"，让 markdown-to-wechat.js 在生成 HTML 时去掉 <a> 标签但保留文字
 * - 同步输出一份过滤报告，方便人工核实
 *
 * 用法：
 *   node validate-links.cjs <markdown文件>
 *   -> 输出 _link-validation.json（详情）和 _link-validation.txt（汇总）
 *   -> 在 markdown 原文里把不可用的链接替换为纯文本，再调用 markdown-to-wechat.js
 */
const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
if (args.length < 1) {
  console.log('用法: node validate-links.cjs <markdown文件> [输出json]');
  process.exit(1);
}

const inputPath = args[0];
const outputJson = args[1] || inputPath.replace(/\.md$/, '_link-validation.json');
const outputTxt = outputJson.replace(/\.json$/, '.txt');
const outputFiltered = inputPath.replace(/\.md$/, '_filtered.md');

if (!fs.existsSync(inputPath)) {
  console.error('文件不存在: ' + inputPath);
  process.exit(1);
}

const markdown = fs.readFileSync(inputPath, 'utf-8');

// 1. 提取所有链接
const linkRegex = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g;
const linkMap = new Map(); // url -> 出现次数
const linkExamples = new Map(); // url -> 第一个链接文本
let m;
while ((m = linkRegex.exec(markdown)) !== null) {
  const text = m[1];
  const url = m[2];
  if (url.includes('mp.weixin.qq.com')) {
    linkMap.set(url, (linkMap.get(url) || 0) + 1);
    if (!linkExamples.has(url)) linkExamples.set(url, text);
  }
}

const uniqueUrls = Array.from(linkMap.keys());
console.log('找到 ' + linkMap.size + ' 个唯一微信链接（共 ' + Array.from(linkMap.values()).reduce((a,b)=>a+b, 0) + ' 次引用）');

// 2. 验证每个链接
const checkUrl = (url) => {
  const r = spawnSync('curl.exe', [
    '-I', '-s',
    '-A', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    '-H', 'Referer: https://mp.weixin.qq.com/',
    '-H', 'Accept-Language: zh-CN,zh;q=0.9',
    '--max-time', '8',
    url
  ], { encoding: 'utf8', timeout: 12000 });

  if (r.error) return { url, status: 0, retKey: '', location: '', isRisk: true, reason: 'curl_error: ' + r.error.message };
  if (r.status !== 0) return { url, status: 0, retKey: '', location: '', isRisk: true, reason: 'curl_exit_' + r.status };

  const out = r.stdout;
  const statusMatch = out.match(/HTTP\/[\d.]+ (\d+)/);
  const status = statusMatch ? parseInt(statusMatch[1]) : 0;
  const locMatch = out.match(/[Ll]ocation:\s*(.+)/);
  const location = locMatch ? locMatch[1].trim() : '';
  const retKeyMatch = out.match(/RetKey:\s*(.+)/);
  const retKey = retKeyMatch ? retKeyMatch[1].trim() : '';

  // 判断标准：
  // - RetKey: 14 = 触发了 appmsgcaptcha 验证码，编辑器会拒绝
  // - HTTP 302/301 重定向到 appmsgcaptcha = 同样会拒绝
  // - HTTP 200 + RetKey: 11 = 正常文章，可以通过
  // - HTTP 200 + 无 RetKey = 通常正常
  const isRisk = retKey === '14' || location.includes('appmsgcaptcha');
  let reason = '';
  if (isRisk) reason = '触发风控验证码 (RetKey:14) - 微信编辑器会拒绝';
  else if (retKey === '11') reason = '正常 (RetKey:11)';
  else if (retKey) reason = '未知RetKey: ' + retKey;
  else reason = '无 RetKey 标记，需要人工确认';

  return { url, status, retKey, location: location.substring(0, 100), isRisk, reason };
};

const results = [];
for (const url of uniqueUrls) {
  const r = checkUrl(url);
  r.count = linkMap.get(url);
  r.exampleText = linkExamples.get(url).substring(0, 50);
  results.push(r);
  const tag = r.isRisk ? '[RISK]' : '[OK]';
  console.log(`${tag} x${r.count} retKey=${r.retKey || '-'} ${url}`);
}

fs.writeFileSync(outputJson, JSON.stringify(results, null, 2), 'utf-8');

// 3. 生成文本报告
const riskUrls = results.filter(r => r.isRisk);
const okUrls = results.filter(r => !r.isRisk);
const report = [];
report.push('=== 微信链接预验证报告 ===');
report.push('文件: ' + inputPath);
report.push('时间: ' + new Date().toISOString());
report.push('总链接数: ' + results.length);
report.push('可用: ' + okUrls.length);
report.push('风险（编辑器会拒绝）: ' + riskUrls.length);
report.push('');
report.push('--- 风险链接（建议从HTML中移除<a>标签，但保留文字）---');
riskUrls.forEach(r => {
  report.push(`  [x${r.count}] ${r.url}`);
  report.push(`    原因: ${r.reason}`);
  report.push(`    出现位置示例: ${r.exampleText}`);
});
report.push('');
report.push('--- 正常链接 ---');
okUrls.forEach(r => {
  report.push(`  [x${r.count}] ${r.url}`);
});
fs.writeFileSync(outputTxt, report.join('\n'), 'utf-8');
console.log('\n报告已写入:');
console.log('  ' + outputJson);
console.log('  ' + outputTxt);

// 4. 生成过滤后的 markdown：把风险链接的 [text](url) 替换为 text
const riskUrlSet = new Set(riskUrls.map(r => r.url));
let filtered = markdown;
let replaced = 0;
filtered = filtered.replace(linkRegex, (fullMatch, text, url) => {
  if (url.includes('mp.weixin.qq.com') && riskUrlSet.has(url)) {
    replaced++;
    return text; // 去掉链接括号，保留文字
  }
  return fullMatch;
});
fs.writeFileSync(outputFiltered, filtered, 'utf-8');
console.log('\n过滤后的 markdown 已写入: ' + outputFiltered);
console.log('替换链接数: ' + replaced + ' / ' + Array.from(linkMap.values()).reduce((a,b)=>a+b, 0));
console.log('\n下一步：用过滤后的 markdown 调用 markdown-to-wechat.js 生成微信HTML');
console.log('  node markdown-to-wechat.js ' + outputFiltered);
