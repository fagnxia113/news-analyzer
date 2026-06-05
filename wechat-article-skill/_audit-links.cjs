const fs = require('fs');

const markdown = fs.readFileSync('d:\\Amateurinterests\\微信公众号分析\\output\\算力，数据中心，AI动态日报0605-18.md', 'utf-8');
const collectedRaw = JSON.parse(fs.readFileSync('d:\\Amateurinterests\\微信公众号分析\\output\\collected-articles-0605.json', 'utf-8'));
const collected = collectedRaw.articles || collectedRaw;

const reportUrls = new Set();
const re = /\[([^\]]*)\]\((https?:\/\/mp\.weixin\.qq\.com\/[^\s)]+)\)/g;
let m;
while ((m = re.exec(markdown)) !== null) {
  reportUrls.add(m[2]);
}

const collectedUrls = new Set();
collected.forEach(a => { if (a.link) collectedUrls.add(a.link); });

// 找出AI幻觉的链接以及它们在报告中被引用的位置
const hallucinated = [];
reportUrls.forEach(u => {
  if (!collectedUrls.has(u)) hallucinated.push(u);
});

console.log('=== AI幻觉的链接 ===');
hallucinated.forEach(u => {
  console.log('\n--- ' + u + ' ---');
  // 在markdown中查找引用
  const lines = markdown.split('\n');
  lines.forEach((line, i) => {
    if (line.includes(u)) {
      console.log(`  L${i+1}: ${line.substring(0, 120)}`);
    }
  });
});

console.log('\n=== 报告链接数 vs 收集的文章数 ===');
console.log('报告中的唯一链接: ' + reportUrls.size);
console.log('收集的文章数: ' + collectedUrls.size);
console.log('AI幻觉的链接: ' + hallucinated.length);
console.log('被复用>1次的链接:');
const urlCount = {};
const re2 = /\[([^\]]*)\]\((https?:\/\/mp\.weixin\.qq\.com\/[^\s)]+)\)/g;
while ((m = re2.exec(markdown)) !== null) {
  urlCount[m[2]] = (urlCount[m[2]] || 0) + 1;
}
Object.entries(urlCount).filter(([_, c]) => c > 1).forEach(([u, c]) => {
  console.log(`  [x${c}] ${u}`);
});
