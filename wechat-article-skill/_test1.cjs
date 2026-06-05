const https = require('https');

const url = 'https://mp.weixin.qq.com/s/yHGdf7TQ0fgLhu46zuUtig';
console.log('Testing:', url);

https.get(url, { headers: { 'User-Agent': 'Mozilla/5.0' } }, (res) => {
  console.log('Status:', res.statusCode);
  console.log('Location:', res.headers.location || '(none)');
  console.log('RetKey:', res.headers['retkey'] || '(none)');
  let body = '';
  res.on('data', (chunk) => { body += chunk; if (body.length > 1000) res.destroy(); });
  res.on('end', () => {
    console.log('Body length:', body.length);
    console.log('Body sample:', body.substring(0, 200));
  });
}).on('error', (e) => console.log('Error:', e.message));
