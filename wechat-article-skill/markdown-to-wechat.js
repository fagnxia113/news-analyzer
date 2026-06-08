#!/usr/bin/env node

import fs from 'fs';
import path from 'path';

const THEME_COLOR = '#059669';
const TEXT_COLOR = '#0F172A';
const GRAY_TEXT = '#475569';
const CARD_BG = '#F8FAFC';
const BORDER_COLOR = '#E2E8F0';
const LIGHT_BG = '#F1F5F9';

const SANS_FONT = 'system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif';

function convertBold(text) {
  return text.replace(/\*\*([^*]+)\*\*/g, '<strong style="color: #059669; font-weight: 600;">$1</strong>');
}

function parseMarkdownSections(markdown) {
  const lines = markdown.split('\n');
  const sections = {
    intro: '',
    judgment: {
      thesis: '',
      bullets: []
    },
    audience: {
      bullets: []
    },
    topNews: [],
    quickScan: {}
  };

  let currentSection = null;
  let currentTopNews = null;
  let currentCategory = null;
  let inTable = false;
  let tableRows = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    if (line.startsWith('> 💡') || line.startsWith('>**') || line.startsWith('>')) {
      let cleanLine = line.replace(/^>\s*/, '').replace(/\*\*/g, '').replace(/^💡\s*/, '').trim();
      if (cleanLine) {
        sections.intro += (sections.intro ? '<br><br>' : '') + cleanLine;
      }
      continue;
    }

    if (line.startsWith('## 今日判断')) {
      currentSection = 'judgment';
      continue;
    }

    if (line.startsWith('## 今天对谁有用')) {
      currentSection = 'audience';
      continue;
    }

    if (line.startsWith('## 🔥 今日重点') || line.startsWith('## 今日重点') || line.startsWith('## 🔥 今日主线') || line.startsWith('## 今日主线')) {
      currentSection = 'topNews';
      continue;
    }

    if (line.startsWith('## ⚡ 一分钟速览') || line.startsWith('## 一分钟速览')) {
      currentSection = 'quickScan';
      continue;
    }

    if (currentSection === 'judgment') {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('---')) continue;

      const cleanLine = convertBold(trimmed.replace(/^[-*]\s*/, '').replace(/^一句话[：:]\s*/, '').trim());
      if (!cleanLine) continue;

      if (trimmed.startsWith('-') || trimmed.startsWith('*')) {
        sections.judgment.bullets.push(cleanLine);
      } else if (!sections.judgment.thesis) {
        sections.judgment.thesis = cleanLine;
      } else {
        sections.judgment.bullets.push(cleanLine);
      }
      continue;
    }

    if (currentSection === 'audience') {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('---')) continue;

      const cleanLine = convertBold(trimmed.replace(/^[-*]\s*/, '').trim());
      if (!cleanLine) continue;

      if (trimmed.startsWith('-') || trimmed.startsWith('*')) {
        sections.audience.bullets.push(cleanLine);
      }
      continue;
    }

    if (currentSection === 'topNews') {
      if (line.startsWith('### ')) {
        if (currentTopNews) sections.topNews.push(currentTopNews);
        currentTopNews = {
          title: line.replace('### ', '').replace(/^\d+\.\s*/, '').trim(),
          link: '', links: [], summary: '', reason: '', insight: '', beneficiary: '', pressure: '', watch: '',
          _pendingField: null
        };
        continue;
      }

      if (!currentTopNews) continue;

      // 支持多个原文链接（用 | 分隔），兼容 [原文链接] 和 [链接] 两种写法
      const linkMatch = line.match(/\[(?:原文链接|链接)\]\(([^)]+)\)/g);
      if (linkMatch) {
        linkMatch.forEach(m => {
          const url = m.match(/\[(?:原文链接|链接)\]\(([^)]+)\)/);
          if (url) {
            if (!currentTopNews.link) currentTopNews.link = url[1];
            currentTopNews.links.push(url[1]);
          }
        });
        continue;
      }

      // 兼容"标题和内容在同一行"与"标题独占一行"两种写法
      // 如果当前行是 **xxx** 格式的字段标题（独占一行，后面没有内容）
      // 则记录 _pendingField，等下一行内容到来时再赋值
      const fieldPatterns = [
        { prefix: '**发生了什么', field: 'summary', isSummary: true },
        { prefix: '**为什么重要', field: 'reason' },
        { prefix: '**这意味着', field: 'insight' },
        { prefix: '**受益方', field: 'beneficiary' },
        { prefix: '**谁该关注', field: 'beneficiary' },
        { prefix: '**承压方', field: 'pressure' },
        { prefix: '**后续观察', field: 'watch' },
        { prefix: '**下一步看什么', field: 'watch' },
      ];

      let matchedField = null;
      for (const fp of fieldPatterns) {
        if (line.startsWith(fp.prefix)) {
          matchedField = fp;
          break;
        }
      }

      if (matchedField) {
        const text = line.replace(/\*\*[^*]+\*\*[：:]*\s*/, '').trim();
        if (text) {
          // 标题和内容在同一行
          if (matchedField.isSummary) {
            currentTopNews.summary += (currentTopNews.summary ? '<br><br>' : '') + convertBold(text);
          } else {
            currentTopNews[matchedField.field] = text;
          }
          currentTopNews._pendingField = null;
        } else {
          // 标题独占一行，等下一行赋值
          currentTopNews._pendingField = matchedField;
        }
        continue;
      }

      // 如果有 pendingField，将当前行内容赋给该字段
      if (currentTopNews._pendingField) {
        const pf = currentTopNews._pendingField;
        const text = line.trim();
        if (text && !text.startsWith('---')) {
          if (pf.isSummary) {
            currentTopNews.summary += (currentTopNews.summary ? '<br><br>' : '') + convertBold(text);
          } else {
            currentTopNews[pf.field] = text;
          }
        }
        currentTopNews._pendingField = null;
        continue;
      }

      if (line.trim() && !line.startsWith('---') && !line.startsWith('[')) {
        currentTopNews.summary += (currentTopNews.summary ? '<br><br>' : '') + convertBold(line.trim());
      }
    }

    if (currentSection === 'quickScan') {
      if (line.startsWith('### ')) {
        if (currentCategory && tableRows.length > 0) sections.quickScan[currentCategory] = tableRows;
        currentCategory = line.replace('### ', '').trim();
        tableRows = [];
        inTable = false;
        continue;
      }

      // 如果没有 ### 分类标题，使用默认分类名
      if (!currentCategory && line.startsWith('|')) {
        currentCategory = '速览';
      }

      if (line.startsWith('|') && line.includes('动态概要')) {
        inTable = true;
        continue;
      }
      if (line.startsWith('|') && line.includes('---')) continue;

      if (line.startsWith('|') && inTable) {
        const cells = line.split('|').filter(c => c.trim()).map(c => c.trim());
        if (cells.length >= 3) {
          tableRows.push({ summary: cells[0], impact: cells[1], link: cells[2] });
        }
        continue;
      }

      if (!line.startsWith('|') && inTable && currentCategory) inTable = false;
    }
  }

  if (currentTopNews) sections.topNews.push(currentTopNews);
  if (currentCategory && tableRows.length > 0) sections.quickScan[currentCategory] = tableRows;

  return sections;
}

function generateWechatHTML(sections, linkWhitelist = null) {
  // 链接白名单过滤：
  // 微信编辑器保存草稿时会验证正文中每个 <a href="..."> 链接
  // 如果链接对应的文章跟正文内容不匹配（例如AI幻觉出来的URL），
  // 或者触发了微信风控/限流，会返回"请勿输入不合法的消息图文链接"
  //
  // 用法：从 collected-articles.json 读取所有真实链接作为白名单
  // 报告中引用但不在白名单的链接，会被自动转成纯文字（保留可读性，去掉可点击性）
  // 这样可以最大程度保留合法链接，避免因为1-2个坏链接毁掉整篇草稿
  const isLinkValid = (url) => {
    if (!url) return false;
    if (!linkWhitelist) return true; // 无白名单时，全部通过（保持向后兼容）
    return linkWhitelist.has(url);
  };

  let html = '';

  html += `<div style="font-family: ${SANS_FONT}; letter-spacing: 0.5px; color: ${TEXT_COLOR}; background-color: #ffffff; padding: 12px 16px;">`;

  if (sections.judgment.thesis || sections.judgment.bullets.length > 0) {
    html += `<section style="margin: 0 0 12px 0; padding: 16px 18px; background-color: ${CARD_BG}; border-left: 3px solid ${THEME_COLOR};">`;
    html += `<p style="margin: 0 0 8px 0; font-size: 15px; font-weight: bold; color: ${THEME_COLOR}; letter-spacing: 1px;">今日判断</p>`;
    if (sections.judgment.thesis) {
      html += `<p style="margin: 0 0 10px 0; font-size: 17px; line-height: 1.75; color: ${TEXT_COLOR}; font-weight: 600; text-align: justify;">${sections.judgment.thesis}</p>`;
    }
    if (sections.judgment.bullets.length > 0) {
      sections.judgment.bullets.slice(0, 3).forEach((bullet, idx) => {
        html += `<p style="margin: 0 0 ${idx < Math.min(sections.judgment.bullets.length, 3) - 1 ? '10px' : '0'} 0; font-size: 16px; line-height: 1.8; color: ${TEXT_COLOR}; text-align: justify;"><strong style="color: ${THEME_COLOR};">${idx + 1}.</strong> ${bullet}</p>`;
      });
    }
    html += `</section>`;
    // 今日判断块结束后留 2 行空行
    html += `<p style="margin: 0; padding: 0; line-height: 1.6; font-size: 14px;">&nbsp;</p>`;
    html += `<p style="margin: 0; padding: 0; line-height: 1.6; font-size: 14px;">&nbsp;</p>`;
  }

  // 今天对谁有用
  if (sections.audience.bullets.length > 0) {
    html += `<section style="margin: 0 0 12px 0; padding: 14px 18px; background-color: ${CARD_BG}; border-left: 3px solid #6366F1;">`;
    html += `<p style="margin: 0 0 8px 0; font-size: 15px; font-weight: bold; color: #6366F1; letter-spacing: 1px;">今天对谁有用</p>`;
    sections.audience.bullets.forEach((bullet, idx) => {
      html += `<p style="margin: 0 0 ${idx < sections.audience.bullets.length - 1 ? '6px' : '0'} 0; font-size: 15px; line-height: 1.7; color: ${GRAY_TEXT}; text-align: justify;">${idx + 1}. ${bullet}</p>`;
    });
    html += `</section>`;
    html += `<p style="margin: 0; padding: 0; line-height: 1.6; font-size: 14px;">&nbsp;</p>`;
    html += `<p style="margin: 0; padding: 0; line-height: 1.6; font-size: 14px;">&nbsp;</p>`;
  }

  html += `<p style="margin: 24px 0 16px 0; font-size: 18px; font-weight: bold; color: ${THEME_COLOR}; letter-spacing: 2px; border-left: 4px solid ${THEME_COLOR}; padding-left: 12px;">今日主线</p>`;

  sections.topNews.forEach((news, idx) => {
    const num = String(idx + 1).padStart(2, '0');

    html += `<section style="margin-bottom: 28px;">`;

    html += `<p style="margin: 0 0 8px 0; font-size: 17px; font-weight: bold; color: ${TEXT_COLOR}; line-height: 1.5;">`;
    html += `<strong style="color: ${THEME_COLOR};">${num}.</strong> ${news.title}`;
    html += `</p>`;

    if (news.link || news.links.length > 0) {
      html += `<p style="margin: 0 0 10px 0;">`;
      const linksToRender = news.links.length > 0 ? news.links : (news.link ? [news.link] : []);
      linksToRender.forEach((lnk, li) => {
        if (isLinkValid(lnk)) {
          html += `<a href="${lnk}" target="_blank" linktype="text" data-linktype="2" textvalue="" class="mp_article_text_link" style="font-size: 12px; color: ${THEME_COLOR}; text-decoration: none;">原文链接${linksToRender.length > 1 ? (li + 1) : ''} →</a>`;
        } else {
          html += `<span style="font-size: 12px; color: #94A3B8;">原文链接${linksToRender.length > 1 ? (li + 1) : ''}（无可用源）</span>`;
        }
        if (li < linksToRender.length - 1) {
          html += `<span style="font-size: 12px; color: #CBD5E1; margin: 0 4px;">|</span>`;
        }
      });
      html += `</p>`;
    }

    html += `<p style="margin: 0 0 12px 0; font-size: 15px; line-height: 1.85; color: ${TEXT_COLOR}; text-align: justify;">${news.summary}</p>`;

    if (news.reason || news.insight || news.beneficiary || news.pressure || news.watch) {
      html += `<section style="margin-top: 12px; padding: 12px 14px; background-color: ${LIGHT_BG}; border-left: 3px solid ${THEME_COLOR};">`;

      if (news.reason) {
        html += `<p style="margin: 0 0 8px 0; font-size: 13px; line-height: 1.6; color: ${GRAY_TEXT};">`;
        html += `<strong style="color: ${THEME_COLOR}; font-size: 12px;">为什么重要：</strong>${news.reason}`;
        html += `</p>`;
      }

      if (news.insight) {
        html += `<p style="margin: 0 0 8px 0; font-size: 13px; line-height: 1.6; color: ${GRAY_TEXT};">`;
        html += `<strong style="color: ${THEME_COLOR}; font-size: 12px;">这意味着：</strong>${news.insight}`;
        html += `</p>`;
      }

      if (news.beneficiary) {
        html += `<p style="margin: 0 0 8px 0; font-size: 13px; line-height: 1.6; color: ${GRAY_TEXT};">`;
        html += `<strong style="color: ${THEME_COLOR}; font-size: 12px;">谁该关注：</strong>${news.beneficiary}`;
        html += `</p>`;
      }

      if (news.pressure) {
        html += `<p style="margin: 0 0 8px 0; font-size: 13px; line-height: 1.6; color: ${GRAY_TEXT};">`;
        html += `<strong style="color: ${THEME_COLOR}; font-size: 12px;">承压方：</strong>${news.pressure}`;
        html += `</p>`;
      }

      if (news.watch) {
        html += `<p style="margin: 0; font-size: 13px; line-height: 1.6; color: ${GRAY_TEXT};">`;
        html += `<strong style="color: ${THEME_COLOR}; font-size: 12px;">下一步看什么：</strong>${news.watch}`;
        html += `</p>`;
      }

      html += `</section>`;
    }

    html += `</section>`;
  });

  html += `<p style="margin: 24px 0 16px 0; font-size: 18px; font-weight: bold; color: ${THEME_COLOR}; letter-spacing: 2px; border-left: 4px solid ${THEME_COLOR}; padding-left: 12px;">一分钟速览</p>`;

  for (const [category, rows] of Object.entries(sections.quickScan)) {
    html += `<section style="margin-bottom: 12px;">`;
    html += `<table border="0" cellpadding="0" cellspacing="0" width="100%">`;
    // 分类标题行：上下各 10px padding，字体略大，加底色突出分类
    html += `<tr><td colspan="2" style="padding: 10px 0 10px 0; font-size: 15px; font-weight: bold; color: ${THEME_COLOR}; letter-spacing: 1px; text-align: center; background-color: ${LIGHT_BG}; border-top: 1px solid ${BORDER_COLOR}; border-bottom: 1px solid ${BORDER_COLOR};">${category}</td></tr>`;

    rows.forEach((row, idx) => {
      let starCount = 3;
      if (row.impact && row.impact.match(/⭐/g)) {
        starCount = row.impact.match(/⭐/g).length;
      }
      const filledStars = '★'.repeat(starCount);
      const emptyStars = '☆'.repeat(5 - starCount);

      html += `<tr>`;
      // 表格行 padding 从 8px 增加到 13px（更高，更舒适），文字上下居中
      html += `<td width="72%" valign="middle" align="left" style="padding: 13px 12px; font-size: 14px; line-height: 1.7; color: ${TEXT_COLOR}; font-weight: 500; border-bottom: 1px solid ${BORDER_COLOR}; background-color: ${CARD_BG};">`;
      const linkMatch = row.link && row.link.match(/\[链接\]\(([^)]+)\)/);
      const linkUrl = linkMatch ? linkMatch[1] : '';
      if (linkUrl && linkUrl !== '-' && isLinkValid(linkUrl)) {
        html += `<a href="${linkUrl}" target="_blank" linktype="text" data-linktype="2" textvalue="" class="mp_article_text_link" style="color: ${TEXT_COLOR}; text-decoration: none;">${row.summary}</a>`;
      } else {
        // 链接不在白名单或为占位符 → 转成纯文字
        html += row.summary;
      }
      html += `</td>`;
      html += `<td width="28%" align="center" valign="middle" style="padding: 13px 12px; font-size: 15px; letter-spacing: 1px; border-bottom: 1px solid ${BORDER_COLOR}; background-color: ${CARD_BG};">`;
      html += `<span style="color: ${THEME_COLOR};">${filledStars}</span><span style="color: #CBD5E1;">${emptyStars}</span>`;
      html += `</td>`;
      html += `</tr>`;
    });

    html += `</table>`;
    html += `</section>`;
  }

  // 一分钟速览板块结束后留 2 行空行（与"关注引导"区隔开）
  html += `<p style="margin: 0; padding: 0; line-height: 1.6; font-size: 14px;">&nbsp;</p>`;
  html += `<p style="margin: 0; padding: 0; line-height: 1.6; font-size: 14px;">&nbsp;</p>`;

  if (sections.intro) {
    html += `<section style="margin: 28px 0 0 0; padding: 16px 18px; background-color: ${CARD_BG}; border-left: 3px solid ${THEME_COLOR};">`;
    html += `<p style="margin: 0; font-size: 15px; line-height: 1.8; color: ${GRAY_TEXT}; text-align: justify;">${sections.intro}</p>`;
    html += `</section>`;
  }

  html += `<section style="margin-top: 36px; padding: 16px 0; border-top: 1px solid ${BORDER_COLOR}; text-align: center;">`;
  html += `<p style="margin: 0 0 6px 0; font-size: 13px; color: ${GRAY_TEXT};">长按识别二维码添加小编微信</p>`;
  html += `</section>`;

  html += `</div>`;

  // 去掉标签间的空白/换行，防止 ProseMirror 粘贴时解析成空段落
  html = html.replace(/>\s+</g, '><');

  return html;
}

const args = process.argv.slice(2);
if (args.length < 1) {
  console.log('用法: node markdown-to-wechat.js <markdown文件路径> [输出路径] [--whitelist <file>] [--strict-links]');
  console.log('');
  console.log('选项:');
  console.log('  --whitelist <file>    从指定的 collected-articles.json 加载链接白名单');
  console.log('                        白名单外的链接（AI幻觉/失效）会自动转成纯文字');
  console.log('  --strict-links        自动查找同目录下的 collected-articles-*.json 作为白名单');
  console.log('                        不需要手动指定文件路径，适合标准工作流');
  process.exit(1);
}

const inputPath = args[0];
let outputPath = null;
let whitelistPath = null;
let strictLinks = false;
for (let i = 1; i < args.length; i++) {
  if (args[i] === '--whitelist' && i + 1 < args.length) {
    whitelistPath = args[++i];
  } else if (args[i] === '--strict-links') {
    strictLinks = true;
  } else if (!outputPath) {
    outputPath = args[i];
  }
}
outputPath = outputPath || inputPath.replace(/\.md$/, '.wechat.html');

if (!fs.existsSync(inputPath)) {
  console.error(`文件不存在: ${inputPath}`);
  process.exit(1);
}

// --strict-links: 自动查找 collected-articles-*.json
if (strictLinks && !whitelistPath) {
  const outputDir = path.dirname(inputPath);
  const collectedFiles = fs.readdirSync(outputDir)
    .filter(f => f.match(/^collected-articles-.*\.json$/))
    .sort();
  if (collectedFiles.length > 0) {
    whitelistPath = path.join(outputDir, collectedFiles[collectedFiles.length - 1]);
    console.log(`--strict-links: 自动使用 ${collectedFiles[collectedFiles.length - 1]}`);
  } else {
    console.log('--strict-links: 未找到 collected-articles-*.json，所有链接将保留');
  }
}

let linkWhitelist = null;
if (whitelistPath) {
  if (!fs.existsSync(whitelistPath)) {
    console.error(`白名单文件不存在: ${whitelistPath}`);
    process.exit(1);
  }
  try {
    const raw = JSON.parse(fs.readFileSync(whitelistPath, 'utf-8'));
    const arr = raw.articles || raw;
    linkWhitelist = new Set();
    arr.forEach(a => { if (a.link) linkWhitelist.add(a.link); });
    console.log(`✓ 已加载链接白名单: ${linkWhitelist.size} 个合法链接（来自 ${whitelistPath}）`);
  } catch (e) {
    console.error('白名单加载失败: ' + e.message);
    process.exit(1);
  }
}

try {
  const markdown = fs.readFileSync(inputPath, 'utf-8');
  const sections = parseMarkdownSections(markdown);
  const html = generateWechatHTML(sections, linkWhitelist);

  // 统计白名单外的链接（用于提示）
  if (linkWhitelist) {
    const allReportUrls = new Set();
    const re = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g;
    let mm;
    while ((mm = re.exec(markdown)) !== null) {
      if (mm[2].includes('mp.weixin.qq.com')) allReportUrls.add(mm[2]);
    }
    const rejected = [...allReportUrls].filter(u => !linkWhitelist.has(u));
    if (rejected.length > 0) {
      console.log(`⚠ 报告引用了 ${rejected.length} 个不在白名单的链接（已转成纯文字）:`);
      rejected.forEach(u => console.log(`  - ${u}`));
    } else {
      console.log('✓ 报告引用的所有链接都在白名单内');
    }
  }

  // 包裹完整的HTML文档结构，声明charset避免HTTP服务器未指定编码时乱码
  const fullHtml = `<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"></head><body>${html}</body></html>`;

  fs.writeFileSync(outputPath, fullHtml, 'utf-8');
  console.log(`微信HTML排版已生成: ${outputPath}`);

  // 生成 ASCII 别名文件（避免中文文件名在 HTTP 服务器中的编码问题）
  const outputDir = path.dirname(outputPath);
  const aliasPath = path.join(outputDir, 'daily.html');
  fs.writeFileSync(aliasPath, fullHtml, 'utf-8');
  console.log(`ASCII别名已生成: ${aliasPath}`);
} catch (error) {
  console.error(`错误: ${error.message}`);
}
