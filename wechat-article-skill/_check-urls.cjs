const { execSync, spawnSync } = require('child_process');
const fs = require('fs');

const urls = JSON.parse(fs.readFileSync('..\\output\\_unique-urls.json', 'utf8'));

const checkUrl = (url) => {
  const r = spawnSync('curl.exe', [
    '-I',  // 只取响应头
    '-s',   // 静默
    '-A', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    '-H', 'Referer: https://mp.weixin.qq.com/',
    '-H', 'Accept-Language: zh-CN,zh;q=0.9',
    '--max-time', '8',
    url
  ], { encoding: 'utf8', timeout: 10000 });

  if (r.error) return { url, error: r.error.message };
  if (r.status !== 0) return { url, error: 'curl exit ' + r.status, stderr: r.stderr };

  const out = r.stdout;
  const statusMatch = out.match(/HTTP\/[\d.]+ (\d+)/);
  const status = statusMatch ? parseInt(statusMatch[1]) : 0;
  const locMatch = out.match(/[Ll]ocation:\s*(.+)/);
  const location = locMatch ? locMatch[1].trim() : '';
  const retKeyMatch = out.match(/RetKey:\s*(.+)/);
  const retKey = retKeyMatch ? retKeyMatch[1].trim() : '';
  const isCaptcha = location.includes('appmsgcaptcha') || retKey === '14';

  return { url, status, location, retKey, isCaptcha };
};

const results = [];
for (const u of urls) {
  const r = checkUrl(u.url);
  r.count = u.count;
  results.push(r);
  const tag = r.isCaptcha ? '[CAPTCHA]' : (r.error ? '[ERR]' : '[OK]');
  console.log(`${tag} x${r.count} status=${r.status || '?'} retKey=${r.retKey || '-'} ${r.url}`);
}

fs.writeFileSync('..\\output\\_url-check.json', JSON.stringify(results, null, 2), 'utf8');

// 汇总
const captchaUrls = results.filter(r => r.isCaptcha);
const okUrls = results.filter(r => !r.isCaptcha && !r.error);
console.log('\n=== 总结 ===');
console.log('测试总数: ' + results.length);
console.log('[CAPTCHA] ' + captchaUrls.length + ' (编辑器会拒绝)');
console.log('[OK] ' + okUrls.length);
console.log('\n问题链接:');
captchaUrls.forEach(r => console.log('  [x' + r.count + '] ' + r.url));
