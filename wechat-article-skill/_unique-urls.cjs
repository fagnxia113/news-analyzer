const fs = require('fs');
const links = JSON.parse(fs.readFileSync('..\\output\\_links-list.json', 'utf8'));

// Group by URL
const urlCounts = {};
links.forEach(l => {
  urlCounts[l.href] = (urlCounts[l.href] || 0) + 1;
});

const uniqueUrls = Object.keys(urlCounts);
const urlList = uniqueUrls.map(url => ({ url, count: urlCounts[url] }));

urlList.sort((a, b) => b.count - a.count);

console.log('Total links:', links.length);
console.log('Unique URLs:', uniqueUrls.length);
console.log('\nUnique URLs (sorted by frequency):');
urlList.forEach((u, i) => console.log(`${i + 1}. [x${u.count}] ${u.url}`));

fs.writeFileSync('..\\output\\_unique-urls.json', JSON.stringify(urlList, null, 2), 'utf8');
