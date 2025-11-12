<template>
  <div class="articles-page">
    <div class="articles-header">
      <div class="articles-count">共 {{ articles.length }} 篇文章</div>
      <div class="articles-actions">
        <select v-model="selectedFeedId" @change="loadArticles" class="feed-select">
          <option value="">全部订阅源</option>
          <option v-for="feed in feeds" :key="feed.id" :value="feed.id">
            {{ feed.mp_name }}
          </option>
        </select>
        <button class="btn btn-secondary" @click="loadArticles">🔄 刷新</button>
        <button class="btn btn-secondary" @click="toggleSelectAll">
          {{ allSelected ? '取消全选' : '全选' }}
        </button>
        <button
          class="btn btn-primary"
          @click="startAnalysis"
          :disabled="selectedArticles.length === 0 || analyzing"
        >
          <span v-if="analyzing" class="loading-spinner"></span>
          {{ getAnalysisButtonText() }}
        </button>
      </div>
    </div>
    
    <div class="articles-list">
      <div v-for="article in articles" :key="article.id" class="article-item" :class="{ 'selected': selectedArticles.includes(article.id) }">
        <div class="article-checkbox">
          <input 
            type="checkbox" 
            :checked="selectedArticles.includes(article.id)"
            @change="toggleArticleSelection(article.id)"
          />
        </div>
        
        <!-- 文章封面 -->
        <img v-if="article.source_type === 'WeChat' && article.pic_url" :src="article.pic_url" :alt="article.title" class="article-cover" />
        <div v-else class="article-cover placeholder">
          {{ article.source_type === 'RSS' ? '📡' : '📄' }}
        </div>
        
        <div class="article-content">
          <h4 class="article-title" @click="openArticle(article)">{{ article.title }}</h4>
          <div class="article-meta">
            <span class="article-source" :class="`source-${article.source_type.toLowerCase()}`">
              {{ article.source_type === 'RSS' ? '📡 RSS' : '📱 微信' }}
            </span>
            <span class="article-time">{{ formatTime(article.publish_time) }}</span>
            <span class="article-id">ID: {{ article.id.slice(0, 8) }}...</span>
          </div>
          <!-- RSS文章额外信息 -->
          <div v-if="article.source_type === 'RSS' && article.author" class="article-author">
            作者: {{ article.author }}
          </div>
        </div>
        
        <div class="article-actions">
          <button class="btn btn-secondary btn-small" @click="openArticle(article)">
            打开
          </button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="articles.length === 0" class="empty-state">
      <div class="empty-icon">📭</div>
      <div class="empty-title">暂无文章</div>
      <div class="empty-description">
        请先添加订阅源并刷新文章
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Article, WeChatFeed, AnalysisRequest } from '../types'
import { formatUtcTimestamp } from '../utils/timeUtils'

const articles = ref<Article[]>([])
const feeds = ref<WeChatFeed[]>([])
const selectedFeedId = ref('')
const selectedArticles = ref<string[]>([])
const analyzing = ref(false)

const allSelected = computed(() => {
  const articlesList = articles.value || []
  return articlesList.length > 0 && selectedArticles.value.length === articlesList.length
})

onMounted(() => {
  loadFeeds()
  loadArticles()
})

const loadFeeds = async () => {
  try {
    feeds.value = await invoke<WeChatFeed[]>('get_all_feeds')
  } catch (error) {
    console.error('加载订阅源失败:', error)
  }
}

const loadArticles = async () => {
  try {
    if (selectedFeedId.value) {
      articles.value = await invoke<Article[]>('get_feed_articles', {
        feedId: selectedFeedId.value,
        limit: null
      })
    } else {
      articles.value = await invoke<Article[]>('get_all_articles', {
        limit: null
      })
    }
  } catch (error) {
    console.error('加载文章失败:', error)
    // 清空数组以显示空状态
    articles.value = []
  }
}

const openArticle = (article: Article) => {
  console.log('=== 点击文章事件触发 ===')
  console.log('文章:', article)
  
  let url: string
  if (article.source_type === 'WeChat') {
    url = `https://mp.weixin.qq.com/s/${article.id}`
  } else {
    url = article.url
  }
  
  console.log('构造的URL:', url)
  
  // 显示确认对话框
  const confirmed = confirm(`是否打开文章？\n\n标题: ${article.title}\n来源: ${article.source_type === 'RSS' ? 'RSS订阅源' : '微信公众号'}\n链接: ${url}`)
  
  if (confirmed) {
    console.log('用户确认打开链接')
    // 直接使用 window.open 打开外部链接
    try {
      const newWindow = window.open(url, '_blank')
      console.log('window.open 结果:', newWindow)
      if (!newWindow) {
        console.error('弹窗被阻止，尝试其他方式')
        // 备用方案：复制到剪贴板
        navigator.clipboard.writeText(url).then(() => {
          alert('链接已复制到剪贴板: ' + url)
        }).catch(err => {
          console.error('复制到剪贴板失败:', err)
          alert('请手动复制链接: ' + url)
        })
      } else {
        console.log('链接成功打开')
      }
    } catch (error) {
      console.error('打开链接失败:', error)
      alert('无法打开链接，请手动复制: ' + url)
    }
  } else {
    console.log('用户取消打开链接')
  }
}

const toggleSelectAll = () => {
  if (allSelected.value) {
    selectedArticles.value = []
  } else {
    selectedArticles.value = (articles.value || []).map(article => article.id)
  }
}

const toggleArticleSelection = (articleId: string) => {
  const index = selectedArticles.value.indexOf(articleId)
  if (index > -1) {
    selectedArticles.value.splice(index, 1)
  } else {
    selectedArticles.value.push(articleId)
  }
}

const startAnalysis = async () => {
  if (selectedArticles.value.length === 0) {
    showNotification('请先选择要分析的文章', 'warning')
    return
  }

  try {
    analyzing.value = true

    // 显示开始分析的消息
    showSuccess(`开始分析 ${selectedArticles.value.length} 篇文章，请在分析结果页面查看进度...`)

    // 获取默认提示词模板
    const defaultTemplate = await invoke<any>('get_default_prompt_template')
    if (!defaultTemplate) {
      showError('请先在设置页面配置提示词模板')
      return
    }

    // 调用后端分析API（使用新的参数格式）
    const taskId = await invoke<string>('start_analysis', {
      articleIds: selectedArticles.value
    })

    if (taskId) {
      console.log('分析任务已启动:', taskId)

      // 显示详细的分析开始信息
      showAnalysisProgressInfo(selectedArticles.value.length, taskId)

      // 延迟跳转到分析结果页面，让用户看到提示信息
      setTimeout(() => {
        // 触发页面切换事件，让父组件跳转到分析结果页面
        const event = new CustomEvent('switchToAnalysis', {
          detail: { taskId, articlesCount: selectedArticles.value.length }
        })
        window.dispatchEvent(event)
      }, 1500) // 1.5秒后跳转

    } else {
      showError('启动分析失败：未能获取任务ID')
    }

  } catch (error) {
    console.error('启动分析失败:', error)
    showError(`启动分析失败: ${error}`)
  } finally {
    // 延迟重置分析状态，给用户时间看到进度提示
    setTimeout(() => {
      analyzing.value = false
    }, 2000)
  }
}

// 显示分析进度信息
const showAnalysisProgressInfo = (articleCount: number, taskId: string) => {
  const message = `
📊 分析任务已启动！

• 任务ID: ${taskId.slice(0, 8)}...
• 文章数量: ${articleCount} 篇
• 预计时间: ${articleCount * 1} 分钟
• 状态: 正在初始化...

即将跳转到分析结果页面查看实时进度
  `.trim()

  // 如果支持自定义通知，使用详细通知
  if (window.app && window.app.$notify) {
    window.app.$notify.success(message, { timeout: 5000 })
  } else {
    alert(message)
  }
}

// 获取分析按钮文本
const getAnalysisButtonText = () => {
  if (analyzing.value) {
    return '正在启动分析...'
  } else if (selectedArticles.value.length === 0) {
    return '请先选择文章'
  } else {
    return `开始分析 (${selectedArticles.value.length})`
  }
}

// 显示成功消息
const showSuccess = (message: string) => {
  try {
    // 尝试使用 Vue 通知（如果存在），否则使用 alert
    if (window.app && window.app.$notify) {
      window.app.$notify.success(message)
    } else {
      // 使用 alert 作为备用方案，但添加 try-catch 防止页面崩溃
        alert(message)
    }
  } catch (e) {
      console.error('显示通知失败:', e)
    }
  }
}

// 显示错误消息
const showError = (message: string) => {
  try {
    // 尝试使用 Vue 通知（如果存在），否则使用 alert
    if (window.app && window.app.$notify) {
      window.app.$notify.error(message)
    } else {
      // 使用 alert 作为备用方案，但添加 try-catch 防止页面崩溃
      alert(message)
    }
  } catch (e) {
    console.error('显示错误消息失败:', e)
  }
}

// 显示警告消息
const showNotification = (message: string, type: 'info' | 'warning' | 'error' | 'success' = 'info') => {
  try {
    // 尝试使用 Vue 通知（如果存在），否则使用 alert
    if (window.app && window.app.$notify) {
      window.app.$notify[type](message)
    } else {
      // 使用 alert 作为备用方案，但添加 try-catch 防止页面崩溃
      alert(message)
    }
  } catch (e) {
    console.error('显示通知失败:', e)
  }
}

const formatTime = (timestamp: number) => {
  // 使用统一的时间处理工具函数
  return formatUtcTimestamp(timestamp, 'datetime')
}

</script>

<style scoped>
.articles-page {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #f0f0f0;
  min-height: calc(100vh - 140px);
}

.articles-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  background: #f8f9fa;
  padding: 16px 20px;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.articles-count {
  font-size: 14px;
  font-weight: 600;
  color: #2c3e50;
  display: flex;
  align-items: center;
  gap: 6px;
}

.articles-count::before {
  content: '📚';
  font-size: 16px;
}

.articles-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.feed-select {
  padding: 6px 12px;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  font-size: 12px;
  background: white;
  cursor: pointer;
  min-width: 140px;
  transition: all 0.2s ease;
}

.feed-select:hover {
  border-color: #667eea;
}

.feed-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.btn-secondary {
  background: #f8f9fa;
  color: #495057;
  border: 1px solid #dee2e6;
}

.btn-secondary:hover {
  background: #e9ecef;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #5a6fd8;
}

.btn-primary:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.btn-small {
  padding: 4px 8px;
  font-size: 11px;
}

.articles-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.article-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: white;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.article-item:hover {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.article-cover {
  width: 80px;
  height: 60px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.article-cover.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8f9fa;
  color: #6c757d;
  font-size: 20px;
  border: 1px dashed #dee2e6;
}

.article-content {
  flex: 1;
  min-width: 0;
}

.article-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #2c3e50;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  cursor: pointer;
  transition: color 0.2s ease;
}

.article-title:hover {
  color: #667eea;
}

.article-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: #6c757d;
}

.article-time {
  display: flex;
  align-items: center;
  gap: 3px;
}

.article-time::before {
  content: '🕒';
  font-size: 10px;
}

.article-id {
  display: flex;
  align-items: center;
  gap: 3px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
}

.article-id::before {
  content: '🏷️';
  font-size: 10px;
}

.article-source {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 500;
  font-size: 10px;
}

.source-rss {
  background: rgba(255, 159, 64, 0.1);
  color: #ff9f40;
  border: 1px solid rgba(255, 159, 64, 0.2);
}

.source-wechat {
  background: rgba(52, 211, 153, 0.1);
  color: #34d399;
  border: 1px solid rgba(52, 211, 153, 0.2);
}

.article-author {
  font-size: 11px;
  color: #6c757d;
  margin-top: 4px;
  display: flex;
  align-items: center;
  gap: 3px;
}

.article-author::before {
  content: '✍️';
  font-size: 10px;
}

.article-checkbox {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  margin-right: 6px;
  margin-top: 2px;
}

.article-checkbox input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
  accent-color: #667eea;
}

.article-item.selected {
  border-color: #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.article-item.selected .article-title {
  color: #5a6fd8;
}

.article-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  margin-top: 2px;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #6c757d;
  background: white;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 16px;
  margin-bottom: 8px;
  font-weight: 600;
  color: #495057;
}

.empty-description {
  font-size: 14px;
  color: #6c757d;
  line-height: 1.5;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .articles-page {
    padding: 16px;
    border-radius: 12px;
  }

  .articles-header {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }

  .articles-actions {
    justify-content: center;
  }

  .article-item {
    flex-direction: column;
    gap: 12px;
  }

  .article-cover {
    width: 100%;
    height: 200px;
  }

  .article-checkbox {
    margin-right: 0;
    margin-top: 0;
  }
}

/* Loading spinner animation */
.loading-spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-right: 6px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Enhanced analyzing button state */
.btn-primary:disabled {
  background: #6c757d;
  cursor: not-allowed;
  position: relative;
}

.btn-primary:disabled .loading-spinner {
  animation: spin 1s linear infinite;
}
</style>
