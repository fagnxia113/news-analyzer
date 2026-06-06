---
name: "wechat-editor"
description: "Controls WeChat Official Account backend editor via Chrome DevTools MCP. Invoke when editing/publishing articles on mp.weixin.qq.com, setting titles, pasting content, or saving drafts."
---

# WeChat Official Account Editor Control

This skill provides precise control over the WeChat Official Account (公众号) backend editor using Chrome DevTools MCP tools.

## When to Invoke

- User asks to edit/publish articles on mp.weixin.qq.com
- User needs to set article title in WeChat editor
- User needs to paste formatted content into WeChat editor
- User wants to save draft in WeChat backend
- Any operation involving WeChat Official Account backend editor

## ⚠️ 关键操作顺序（必须遵守！）

> **以下三条规则是操作微信编辑器的生命线，违反任何一条都会导致严重问题！**

1. **必须先设置正文，再设置标题！** 如果先设置标题再设置正文，标题会被正文内容覆盖，导致标题丢失。
2. **必须用 `querySelectorAll('.ProseMirror')[1]` 选择正文编辑器！** 绝不能用 `querySelector('.ProseMirror')`，它返回的是标题编辑器（第一个 `.ProseMirror`），会导致内容粘贴到标题栏。
3. **绝不能用 Ctrl+A 全选！** 微信编辑器中 Ctrl+A 会同时选中标题和正文，粘贴时内容会错位到标题栏。

## Editor Structure

The WeChat backend editor uses **ProseMirror** rich text editor framework. Key elements:

### ⚠️ CRITICAL: Two ProseMirror Elements!

The WeChat editor has **TWO** `.ProseMirror` elements on the page. Using `document.querySelector('.ProseMirror')` will return the **FIRST** one (title editor), NOT the body editor!

1. **Title ProseMirror** (first `.ProseMirror`): `document.querySelectorAll('.ProseMirror')[0]`
   - Parent class: `title-editor__input`
   - This is the visual title editor (ProseMirror-based)
   - Also synced with `#title` textarea

2. **Body ProseMirror** (second `.ProseMirror`): `document.querySelectorAll('.ProseMirror')[1]`
   - Parent class: `view rich_media_content autoTypeSetting24psection`
   - This is the main content editor
   - Placeholder text: "从这里开始写正文"

**To select the body editor, ALWAYS use:**
```javascript
const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
```

**NEVER use `document.querySelector('.ProseMirror')` — it returns the title editor!**

### Title Input
- **Selector**: `#title` (TEXTAREA element, not input)
- **Class**: `frm_input js_title js_counter js_field js_article_title`
- **Location**: Top of the page, separate from body editor
- **Max length**: 64 characters
- **Important**: Title should NOT contain emoji (微信限制)
- **Note**: The visual title editor is a separate ProseMirror element that syncs with this textarea

### Body Editor
- **Selector**: `document.querySelectorAll('.ProseMirror')[1]` (second ProseMirror element)
- **Parent class**: `view rich_media_content autoTypeSetting24psection`
- **Location**: Below the title, main content area
- **Framework**: ProseMirror (complex rich text editor)
- **Placeholder text**: "从这里开始写正文"

### Other Elements
- **Save button**: "保存为草稿" button
- **Publish button**: "发表" button (DO NOT click unless user explicitly asks)
- **Author input**: `#author` (optional, 8 characters max)

## Critical Rules

### 1. ⚠️ CRITICAL: Operation Order (Must Follow!)

**MUST set body content BEFORE setting title!**

If you set title first and then set body content, the title will be overwritten by body content.

**Correct Order**:
1. First: Focus the body editor (second `.ProseMirror`) and insert content
2. Wait: 500ms for the editor to stabilize
3. Then: Set title using title ProseMirror editor and `#title.value`

**WRONG Approaches (will fail)**:
- Setting title first, then body → title gets overwritten
- Using `document.querySelector('.ProseMirror')` → returns title editor, not body

### 2. Title vs Body Recognition

**DO NOT confuse title and body!**

- Title is a separate `<textarea>` element (`#title`)
- Body is a `<div>` element (`.ProseMirror`)
- They are NOT in the same container

**Common Mistake**: Pasting body content into title field because Ctrl+A selected both.

### 3. Setting Title

```javascript
// Step 1: Focus the title ProseMirror editor
const titleEditor = document.querySelectorAll('.ProseMirror')[0];
titleEditor.focus();
await new Promise(r => setTimeout(r, 300));
document.execCommand('selectAll', false, null);
document.execCommand('delete', false, null);
document.execCommand('insertText', false, '标题文字');

// Step 2: Sync the #title textarea
const titleInput = document.querySelector('#title');
titleInput.value = '标题文字';
titleInput.dispatchEvent(new Event('input', { bubbles: true }));
titleInput.dispatchEvent(new Event('change', { bubbles: true }));
```

**Rules for title**:
- No emoji (微信后台会报错)
- Max 64 characters
- Format (双轨标题): `{最劲爆新闻关键词}｜M.D算力日报`
- 前半句踩热搜词/引发好奇，后半句保留日报标识
- Example: `软银砸5900亿建欧洲最大AI算力中心，英伟达杀入PC芯片市场｜6.2算力日报`
- Date should be dynamic (use current date)

### 4. Setting Body Content

The ProseMirror body editor requires special handling. **Always use the SECOND `.ProseMirror` element!**

**⚠️ `document.execCommand('insertHTML')` strips all inline styles and formatting!**
**Use ClipboardEvent paste instead to preserve rich text formatting.**

#### ⚠️ CRITICAL: Cross-Origin Issue

微信后台是 HTTPS 页面，**无法从 HTTPS 页面 fetch 本地 HTTP 服务器（如 `http://localhost:8766`）的内容**，浏览器会因混合内容策略（Mixed Content）阻止请求。

因此，不能在微信编辑器页面内直接 `fetch('http://localhost:8766/...')` 获取 HTML。

#### Method A: Cross-Page Clipboard Paste (Recommended - 实测可行)

**核心思路**：在本地 HTTP 页面打开 HTML → 复制到剪贴板 → 切回微信编辑器 → 粘贴

**Step 1: 启动本地 HTTP 服务器**（在 output 目录下）

```bash
# 在 output 目录启动 HTTP 服务器（端口 8766）
cd output
node -e "const http=require('http'),fs=require('fs'),path=require('path');http.createServer((req,res)=>{const filePath=path.join(__dirname,req.url==='/'?'index.html':req.url);const ext=path.extname(filePath);const ct={'html':'text/html','css':'text/css','js':'application/javascript'}[ext.slice(1)]||'text/plain';fs.readFile(filePath,(err,data)=>{if(err){res.writeHead(404);res.end('Not found');return}res.writeHead(200,{'Content-Type':ct+';charset=utf-8','Access-Control-Allow-Origin':'*'});res.end(data)})}).listen(8766,()=>console.log('Server on 8766'))"
```

**Step 2: 在新标签页打开 HTML 文件**

使用 Chrome DevTools MCP 的 `new_page` 工具打开：
```
http://localhost:8766/daily.html
```

注意：由于中文文件名在 URL 编码中容易出问题，`markdown-to-wechat.js` 会自动生成 `daily.html` 作为 ASCII 别名。如果文件名是中文，需先手动复制：
```bash
copy "output\算力，数据中心，AI动态日报0603.wechat.html" "output\daily.html"
```

**Step 3: 在 HTTP 页面复制 HTML 到剪贴板**

在该 HTTP 页面执行 `evaluate_script`：

```javascript
async () => {
  const body = document.body;
  const range = document.createRange();
  range.selectNodeContents(body.querySelector('div') || body);
  const selection = window.getSelection();
  selection.removeAllRanges();
  selection.addRange(range);

  const divEl = document.querySelector('div');
  const htmlContent = divEl ? divEl.outerHTML : body.innerHTML;

  try {
    const blob = new Blob([htmlContent], { type: 'text/html' });
    const plainBlob = new Blob([document.body.innerText], { type: 'text/plain' });
    const item = new ClipboardItem({
      'text/html': blob,
      'text/plain': plainBlob
    });
    await navigator.clipboard.write([item]);
    return 'Copied to clipboard, HTML length: ' + htmlContent.length;
  } catch (e) {
    return 'Clipboard API failed: ' + e.message;
  }
}
```

**Step 4: 切回微信编辑器页面**

使用 `select_page` 切回微信编辑器所在的标签页。

**Step 5: 清空正文并粘贴**

在微信编辑器页面执行 `evaluate_script`：

```javascript
async () => {
  // 1. 找到正文编辑器（第二个 .ProseMirror）
  const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
  if (!bodyEditor) return 'Body editor not found';

  // 2. 通过 Vue 组件找到 EditorView，清空内容
  let el = bodyEditor;
  let editorView = null;
  while (el) {
    if (el.__vue__ && el.__vue__.__editorView) {
      editorView = el.__vue__.__editorView;
      break;
    }
    el = el.parentElement;
  }

  if (editorView) {
    const tr = editorView.state.tr.delete(0, editorView.state.doc.content.size);
    editorView.dispatch(tr);
    await new Promise(r => setTimeout(r, 300));
  }

  // 3. 从剪贴板读取 HTML 并通过 ClipboardEvent 粘贴
  try {
    const items = await navigator.clipboard.read();
    for (const item of items) {
      if (item.types.includes('text/html')) {
        const blob = await item.getType('text/html');
        const htmlContent = await blob.text();

        const dataTransfer = new DataTransfer();
        dataTransfer.setData('text/html', htmlContent);
        dataTransfer.setData('text/plain', htmlContent);

        bodyEditor.focus();
        const pasteEvent = new ClipboardEvent('paste', {
          bubbles: true,
          cancelable: true,
          clipboardData: dataTransfer
        });
        bodyEditor.dispatchEvent(pasteEvent);

        await new Promise(r => setTimeout(r, 2000));
        return 'Pasted HTML content, length: ' + htmlContent.length;
      }
    }
    return 'No HTML content in clipboard';
  } catch (e) {
    return 'Clipboard read failed: ' + e.message;
  }
}
```

**Step 6: 清除 ProseMirror 粘贴时自动插入的空段落（重要！）**

ProseMirror 在粘贴含 `<section>` 标签的 HTML 时，会自动在 `<section>` 前后插入空段落 `<p><br class="ProseMirror-trailingBreak"></span></p>`，导致首行空行、"今日重点"前后空行等问题。**粘贴后必须清除这些空段落**：

```javascript
() => {
  const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
  if (!bodyEditor) return 'Body editor not found';

  const emptyParagraphs = bodyEditor.querySelectorAll('p');
  let removed = 0;
  emptyParagraphs.forEach(p => {
    const text = p.textContent.trim();
    const hasOnlyBreak = p.querySelector('br.ProseMirror-trailingBreak') || p.querySelector('br');
    // 只删除没有样式（非内容段落）且仅含换行符的空段落
    if (text === '' && hasOnlyBreak && !p.style.cssText.includes('font-size') && !p.style.cssText.includes('margin: 24px')) {
      p.remove();
      removed++;
    }
  });

  return 'Removed ' + removed + ' empty paragraphs';
}
```

#### Method B: Inline HTML Injection (备选 - 无需 HTTP 服务器)

如果不想启动 HTTP 服务器，可以直接将 HTML 内容作为字符串注入。但需要注意：
- 需要在 `evaluate_script` 中将 HTML 作为字符串传递
- 大文件可能导致脚本超时或字符串转义问题
- 适用于内容较短的情况

```javascript
async () => {
  const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
  if (!bodyEditor) return 'Body editor not found';

  // 清空内容
  let el = bodyEditor;
  let editorView = null;
  while (el) {
    if (el.__vue__ && el.__vue__.__editorView) {
      editorView = el.__vue__.__editorView;
      break;
    }
    el = el.parentElement;
  }
  if (editorView) {
    const tr = editorView.state.tr.delete(0, editorView.state.doc.content.size);
    editorView.dispatch(tr);
    await new Promise(r => setTimeout(r, 300));
  }

  // HTML 内容直接内联（注意转义）
  const htmlContent = '...你的HTML内容...';

  const dataTransfer = new DataTransfer();
  dataTransfer.setData('text/html', htmlContent);
  dataTransfer.setData('text/plain', htmlContent);

  bodyEditor.focus();
  const pasteEvent = new ClipboardEvent('paste', {
    bubbles: true,
    cancelable: true,
    clipboardData: dataTransfer
  });
  bodyEditor.dispatchEvent(pasteEvent);

  await new Promise(r => setTimeout(r, 2000));
  return 'Done';
}
```

**DO NOT**:
- 在 HTTPS 页面 fetch HTTP 服务器（混合内容策略会阻止）
- 使用 `document.querySelector('.ProseMirror')` — 返回的是标题编辑器！
- 使用 `editor.innerHTML = ...`（ProseMirror 不会注册变更）
- 使用 `document.execCommand('insertHTML')` 粘贴富文本内容（会丢失所有样式！）
- 使用 Ctrl+A 全选（会同时选中标题和正文，导致粘贴错位）

### 5. Clearing Existing Content

Before pasting new content, use ProseMirror transaction:

```javascript
const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
let el = bodyEditor;
let editorView = null;
while (el) {
  if (el.__vue__ && el.__vue__.__editorView) {
    editorView = el.__vue__.__editorView;
    break;
  }
  el = el.parentElement;
}

if (editorView) {
  const tr = editorView.state.tr.delete(0, editorView.state.doc.content.size);
  editorView.dispatch(tr);
}
```

## Workflow for Publishing Daily Report

### Step 1: Open Editor via "选择已有内容"

点击首页"选择已有内容"会**打开新标签页**（不是弹窗），新标签页同时包含编辑器和文章选择面板。

**⚠️ 关键**：a11y snapshot 无法捕获选择面板中的文章列表项，必须用 `evaluate_script` 操作。且无法可靠检测文章的"选中"状态，因此采用**结果验证**策略：点击后检查编辑器是否加载了内容，没加载就重试。

**完整流程（含重试逻辑）**：

1. 在首页点击"选择已有内容" → 新标签页打开
2. 用 `evaluate_script` 点击 `.publish_list_item` 选择文章
3. 用 `click` 工具点击 a11y snapshot 中的"确定"按钮
4. 等待 2 秒后验证：检查 `#title` 是否有内容
5. 如果标题为空 → 选择未生效，关闭标签页，从步骤 1 重新开始
6. 如果标题有内容 → 成功进入编辑器，继续后续步骤

**选择文章代码**：
```javascript
// 点击第一篇文章（.publish_list_item）
const firstItem = document.querySelector('.publish_list_item');
if (firstItem) {
  firstItem.click();
}
```

**验证加载结果代码**：
```javascript
async () => {
  await new Promise(r => setTimeout(r, 2000));
  const titleInput = document.querySelector('#title');
  const titleValue = titleInput?.value;
  const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
  const bodyText = bodyEditor?.innerText?.substring(0, 50);
  return { titleValue, bodyText, success: !!titleValue };
}
```

**备选方案**：如果"选择已有内容"多次重试仍失败，点击首页"文章"按钮创建新文章（编辑器为空，需手动粘贴全部内容）。

### Step 2: Generate WeChat HTML

1. Run `node markdown-to-wechat.js <markdown_file>` to generate WeChat-compatible HTML
2. The output file will be `<filename>.wechat.html` with inline styles
3. A `daily.html` alias will also be created for easy HTTP serving (avoids Chinese filename encoding issues)
4. Start a local HTTP server in the output directory on port 8766

### Step 3: Set Body Content (BEFORE title!)

1. Open the HTML file in a new browser tab via `http://localhost:8766/daily.html`
2. Copy the HTML content to clipboard using `navigator.clipboard.write()`
3. Switch back to the WeChat editor tab
4. Clear the body editor using ProseMirror transaction
5. Paste the HTML content using `ClipboardEvent` with clipboard data
6. Verify content is in body, NOT in title

### Step 4: Set Title

1. Focus the title editor: `document.querySelectorAll('.ProseMirror')[0]`
2. Select all and delete existing content
3. Use `document.execCommand('insertText', false, '标题文字')` to set title
4. Also update `#title` textarea value and dispatch input event
5. Verify title is set correctly

### Step 5: Save Draft

1. Find "保存为草稿" button
2. Click to save
3. Wait for success message
4. **DO NOT click "发表"** unless user explicitly asks
5. If you get "请勿插入不合法的图文消息链接" error, see Troubleshooting section

### Step 6: Cleanup

1. Stop the local HTTP server on port 8766
2. Close the HTML preview tab
3. Delete temporary files (keep the final .md file)

## Troubleshooting

### Problem: fetch from localhost fails on HTTPS page

**Cause**: Mixed Content Policy — browsers block HTTP requests from HTTPS pages.

**Solution**: Use the Cross-Page Clipboard Paste method (Method A above). Open the HTML in a separate HTTP tab, copy to clipboard, then paste in the HTTPS WeChat editor.

### Problem: Content pasted into title field

**Cause**: Using `document.querySelector('.ProseMirror')` which returns the FIRST (title) ProseMirror element, not the body editor.

**Solution**:
1. Always use `document.querySelectorAll('.ProseMirror')[1]` for the body editor
2. Never use `document.querySelector('.ProseMirror')` — it returns the title editor!

### Problem: ProseMirror not registering content

**Cause**: Direct `innerHTML` assignment doesn't trigger ProseMirror state update.

**Solution**: Use `ClipboardEvent` paste or `insertHTML` command.

### Problem: Title contains emoji and fails

**Cause**: WeChat backend rejects emoji in titles.

**Solution**: Remove all emoji from title before setting.

### Problem: "请勿插入不合法的图文消息链接" when saving draft

**Cause**: WeChat backend validates all `<a>` links when saving. Some links (e.g., deleted articles, non-public articles) are flagged as "不合法". Usually only specific links are problematic, not all.

**Solution (优先)**: Identify and remove the specific broken link from the content before saving. Check which link is causing the error and either:
1. Remove that specific `<a>` tag's href in the editor
2. Or regenerate the HTML without that link

**Solution (应急)**: If you can't identify which link is broken, remove all href attributes before saving. ProseMirror will automatically restore them after saving:

```javascript
async () => {
  const bodyEditor = document.querySelectorAll('.ProseMirror')[1];
  if (!bodyEditor) return 'Body editor not found';
  const links = bodyEditor.querySelectorAll('a[href]');
  let removed = 0;
  links.forEach(a => { a.removeAttribute('href'); removed++; });
  await new Promise(r => setTimeout(r, 500));
  return 'Removed href from ' + removed + ' links';
}
```

### Problem: Chinese filename encoding issues in HTTP server

**Cause**: Chinese characters in filenames get URL-encoded, which the simple Node.js HTTP server may not handle correctly.

**Solution**: Use the `daily.html` alias file instead of the Chinese-named original. The `markdown-to-wechat.js` script should auto-generate this alias.

### Problem: Clipboard API permission denied

**Cause**: The Clipboard API requires a secure context (HTTPS or localhost) and user interaction.

**Solution**: Ensure you're on the HTTP localhost page when writing to clipboard, and on the HTTPS WeChat page when reading from clipboard. Both contexts support the Clipboard API.

### Problem: Chrome DevTools MCP 不可用

**Cause**: Agent 运行环境不支持 Chrome DevTools MCP 工具（如 `evaluate_script`、`take_snapshot`、`click` 等不可用），无法通过脚本自动操作浏览器。

**Solution**: Agent 应告知用户手动完成排版和保存草稿操作，提供以下手动步骤：

1. 浏览器打开 `http://localhost:8766/daily.html`
2. 按 Ctrl+A 选中页面全部内容
3. 按 Ctrl+C 复制
4. 切换到微信编辑器页面
5. **鼠标点击正文区域**（确保光标在正文中，而非标题栏）
6. 按 Ctrl+V 粘贴
7. 手动设置标题（在标题栏输入）
8. 点击"保存为草稿"

**⚠️ 注意**：在微信编辑器中不要用 Ctrl+A 全选，否则会同时选中标题和正文，导致粘贴错位。如需清空正文再粘贴，用鼠标选中正文区域后删除即可。

## MCP Tool Usage

Use Chrome DevTools MCP tools:

- `take_snapshot`: Get page structure, identify elements
- `click`: Click on elements (use uid from snapshot)
- `evaluate_script`: Run JavaScript for complex operations
- `press_key`: Keyboard shortcuts (Ctrl+V, Ctrl+A)
- `navigate_page`: Navigate between pages
- `select_page`: Switch between tabs
- `new_page`: Open new browser tab (for HTTP pages)

## Important Notes

1. **Always verify element before action**: Use `take_snapshot` to confirm correct element
2. **Title and Body are separate**: Never assume they're in same container
3. **ProseMirror needs special handling**: Use `ClipboardEvent` paste (not `insertHTML` which strips styles)
4. **No emoji in title**: WeChat will reject
5. **Save as draft only**: Do NOT publish unless explicitly asked
6. **Date is dynamic**: Use current date for title, not hardcoded
7. **Cross-origin restriction**: Cannot fetch HTTP from HTTPS page — use cross-page clipboard method
8. **Use daily.html alias**: Avoid Chinese filename encoding issues with HTTP server
