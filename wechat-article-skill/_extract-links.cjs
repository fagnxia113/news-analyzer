const fs = require('fs');
const html = fs.readFileSync('..\\output\\daily.html', 'utf8');
const re = /<a href="([^"]+)"[^>]*>([^<]+)<\/a>/g;
let m;
const links = [];
while ((m = re.exec(html)) !== null) {
  links.push({ href: m[1], text: m[2].substring(0, 80) });
}
console.log('Total links:', links.length);
fs.writeFileSync('..\\output\\_links-list.json', JSON.stringify(links, null, 2), 'utf8');
links.forEach((l, i) => console.log((i+1) + '. ' + l.href + ' | ' + l.text));
