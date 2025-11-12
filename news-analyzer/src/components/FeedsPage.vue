<template>
  <div class="feeds-layout">
    <!-- 左侧：订阅源列表 -->
    <div class="feeds-sidebar">
      <div class="feeds-sidebar-header">
        <h3>订阅源</h3>
        <div class="feeds-actions">
          <button class="btn btn-primary btn-small" @click="showAddFeedModal">+ 添加</button>
        </div>
      </div>
      
      <div class="feeds-sidebar-content">
        <div class="feed-list">
          <!-- 全部文章汇总 -->
          <div 
            class="feed-item" 
            :class="{ active: selectedFeedId === 'all' }"
            @click="selectFeed('all')"
          >
            <div class="feed-avatar">📰</div>
            <div class="feed-info">
              <h4>全部文章</h4>
              <div class="feed-meta">
                <span>{{ totalArticles }}篇文章</span> • 
                <span>实时更新</span>
              </div>
            </div>
          </div>
          
          <!-- 各个订阅源 -->
          <div 
            v-for="feed in feeds" 
            :key="feed.id"
            class="feed-item"
            :class="{ active: selectedFeedId === feed.id }"
          >
            <div class="feed-content" @click="selectFeed(feed.id)">
              <div class="feed-avatar">
                <img v-if="feed.mp_cover" :src="feed.mp_cover" :alt="feed.mp_name" />
                <span v-else>📰</span>
              </div>
              <div class="feed-info">
                <h4>{{ feed.mp_name }}</h4>
                <div class="feed-meta">
                  <span>{{ feedArticleCounts[feed.id] || 0 }}篇文章</span> • 
                  <span>{{ formatLastUpdated(feed.updated_at) }}</span>
                </div>
              </div>
            </div>
            <div class="feed-actions">
              <button 
                class="btn-delete" 
                @click.stop="deleteFeed(feed.id, feed.mp_name)"
                title="删除订阅源"
              >
                🗑️
              </button>
            </div>
          </div>
          
          <!-- RSS订阅源 -->
          <div 
            v-for="rssFeed in rssFeeds" 
            :key="rssFeed.id"
            class="feed-item"
            :class="{ active: selectedFeedId === rssFeed.id }"
          >
            <div class="feed-content" @click="selectFeed(rssFeed.id)">
              <div class="feed-avatar">
                <span>🌐</span>
              </div>
              <div class="feed-info">
                <h4>{{ rssFeed.title }}</h4>
                <div class="feed-meta">
                  <span>{{ feedArticleCounts[rssFeed.id] || 0 }}篇文章</span> • 
                  <span>{{ formatLastUpdated(rssFeed.updated_at) }}</span>
                  <span v-if="rssFeed.category" class="feed-category">• {{ rssFeed.category }}</span>
                </div>
              </div>
            </div>
            <div class="feed-actions">
              <button 
                class="btn-delete" 
                @click.stop="deleteRssFeed(rssFeed.id, rssFeed.title)"
                title="删除RSS订阅源"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧：文章列表 -->
    <div class="articles-panel">
      <div class="articles-header">
        <h3>{{ currentFeedName }}</h3>
        <div class="articles-actions">
          <div class="date-filter">
            <select v-model="dateFilter" @change="filterByDate">
              <option value="all">全部时间</option>
              <option value="today">今天</option>
              <option value="yesterday">昨天</option>
              <option value="week">最近7天</option>
              <option value="month">最近30天</option>
            </select>
          </div>
          <button
            class="btn btn-secondary btn-small"
            @click="refreshCurrentFeed"
            :disabled="refreshing"
            :title="selectedFeedId === 'all' ? '刷新所有订阅源' : '刷新当前订阅源'"
          >
            <span v-if="refreshing" class="refresh-spinner"></span>
            🔄 {{ refreshing ? (selectedFeedId === 'all' ? '批量刷新中...' : '刷新中...') : (selectedFeedId === 'all' ? '刷新全部' : '刷新') }}
          </button>
          <button
            v-if="refreshing && selectedFeedId === 'all'"
            class="btn btn-danger btn-small"
            @click="interruptRefresh"
            title="中断刷新任务"
          >
            ⏹️ 中断
          </button>
        </div>
      </div>
      
      <div class="articles-content">
        <div v-if="loading" class="articles-empty">
          <div class="empty-icon">⏳</div>
          <div class="empty-title">加载中...</div>
        </div>
        
        <div v-else-if="articles.length === 0" class="articles-empty">
          <div class="empty-icon">📄</div>
          <div class="empty-title">暂无文章</div>
          <div class="empty-description">该订阅源还没有文章</div>
        </div>
        
        <table v-else class="articles-table">
          <thead>
            <tr>
              <th width="40">
                <input 
                  type="checkbox" 
                  @change="toggleSelectAll"
                  :checked="allSelected"
                >
              </th>
              <th>标题</th>
              <th width="180">发布时间</th>
            </tr>
          </thead>
          <tbody>
            <tr 
              v-for="article in filteredArticles" 
              :key="article.id"
              :class="{ selected: selectedArticles.has(article.id) }"
            >
              <td>
                <input 
                  type="checkbox" 
                  :checked="selectedArticles.has(article.id)"
                  @change="toggleArticleSelection(article.id)"
                >
              </td>
              <td class="article-title-cell">
                <a 
                  href="javascript:void(0)" 
                  class="article-title-link"
                  @click="openArticle(article)"
                >
                  {{ article.title }}
                </a>
              </td>
              <td class="article-time-cell">{{ formatTime(article.publish_time) }}</td>
            </tr>
          </tbody>
        </table>
        
        <div v-if="hasMore" class="load-more-container">
          <button class="btn-load-more" @click="loadMoreArticles" :disabled="loadingMore">
            {{ loadingMore ? '加载中...' : '加载更多' }}
          </button>
        </div>
      </div>
      
      <!-- 分析控制栏 -->
      <div class="articles-analysis-controls">
        <div class="selected-info">
          <span>已选择 {{ selectedArticles.size }} 篇文章</span>
          <button 
            v-if="selectedArticles.size > 0" 
            class="btn btn-secondary btn-small" 
            @click="clearSelection"
            style="margin-left: 12px;"
          >
            清空选择
          </button>
        </div>
        <button 
          class="btn btn-primary" 
          :disabled="selectedArticles.size === 0"
          @click="startAnalysis"
        >
          🚀 开始分析
        </button>
      </div>
    </div>
  </div>

  <!-- 刷新进度弹窗 -->
  <div v-if="showProgressModal" class="modal-overlay">
    <div class="modal progress-modal">
      <div class="modal-header">
        <div class="modal-title">
          🔄 {{ refreshStatus.includes('中断') ? '刷新任务已中断' : '正在刷新文章' }}
        </div>
        <div class="modal-actions">
          <button
            v-if="!refreshStatus.includes('中断')"
            class="btn btn-danger btn-small"
            @click="interruptRefresh"
          >
            ⏹️ 中断任务
          </button>
          <button class="modal-close" @click="closeProgressModal">×</button>
        </div>
      </div>
      <div class="modal-body">
        <div class="progress-info">
          <div class="progress-status">{{ refreshStatus }}</div>
          <div class="progress-details">
            <span>进度: {{ refreshCurrent }} / {{ refreshTotal }}</span>
            <span v-if="refreshTotal > 0">({{ Math.round(refreshProgress) }}%)</span>
            <span v-if="refreshEta" class="progress-eta">预计剩余: {{ refreshEta }}</span>
          </div>
        </div>
        <div class="progress-bar-container">
          <div class="progress-bar" :style="{ width: refreshProgress + '%' }"></div>
        </div>
        
        <!-- 实时日志区域 -->
        <div class="refresh-logs-section">
          <div class="logs-header">
            <span class="logs-title">📋 实时日志</span>
            <button class="btn btn-secondary btn-small" @click="clearLogs">清空</button>
          </div>
          <div class="refresh-logs-container">
            <div 
              v-for="(log, index) in refreshLogs" 
              :key="index"
              class="log-entry"
              :class="`log-${log.level}`"
            >
              <span class="log-time">{{ formatLogTime(log.timestamp) }}</span>
              <span class="log-level">{{ log.level.toUpperCase() }}</span>
              <span class="log-message">
                <span v-if="log.feed_name" class="log-feed-name">[{{ log.feed_name }}]</span>
                {{ log.message }}
              </span>
            </div>
            <div v-if="refreshLogs.length === 0" class="log-empty">
              等待日志输出...
            </div>
          </div>
        </div>
        
        <div class="progress-tips">
          <div>💡 刷新可能需要几分钟时间</div>
          <div v-if="selectedFeedId === 'all'">📱 正在获取所有订阅源的最新文章...</div>
          <div v-else>📱 正在获取当前订阅源的最新文章...</div>
          <div v-if="refreshTotal > 0">⏱️ 预计剩余时间: {{ Math.ceil((refreshTotal - refreshCurrent) * 30) }}秒</div>
        </div>
      </div>
    </div>
  </div>

  <!-- 添加订阅源对话框 -->
  <div v-if="showAddModal" class="modal-overlay" @click="showAddModal = false">
    <div class="modal add-feed-modal" @click.stop>
      <div class="modal-header">
        <div class="modal-title">添加订阅源</div>
        <button class="modal-close" @click="showAddModal = false">×</button>
      </div>
      <div class="modal-body">
        <!-- 订阅源类型选择 -->
        <div class="feed-type-tabs">
          <button 
            class="tab-button" 
            :class="{ active: feedType === 'wechat' }"
            @click="feedType = 'wechat'"
          >
            📱 公众号
          </button>
          <button 
            class="tab-button" 
            :class="{ active: feedType === 'rss' }"
            @click="feedType = 'rss'"
          >
            🌐 RSS
          </button>
        </div>

        <!-- 公众号订阅源 -->
        <div v-if="feedType === 'wechat'" class="feed-type-content">
          <div style="margin-bottom: 16px;">
            <label style="display: block; margin-bottom: 8px; font-weight: 500;">分享链接</label>
            <textarea 
              v-model="wxsLink"
              placeholder="输入公众号文章分享链接，一行一条，如：&#10;https://mp.weixin.qq.com/s/xxxxxx&#10;https://mp.weixin.qq.com/s/xxxxxx"
              style="width: 100%; height: 120px; padding: 8px; border: 1px solid #e0e0e0; border-radius: 6px; resize: vertical; font-family: monospace; font-size: 13px;"
            ></textarea>
          </div>
          <div style="font-size: 12px; color: #666; margin-bottom: 16px;">
            💡 提示：从公众号文章页面复制链接，粘贴即可自动识别公众号信息
          </div>
        </div>

        <!-- RSS订阅源 -->
        <div v-if="feedType === 'rss'" class="feed-type-content">
          <div style="margin-bottom: 16px;">
            <label style="display: block; margin-bottom: 8px; font-weight: 500;">RSS链接</label>
            <input 
              v-model="rssUrl"
              type="url"
              placeholder="输入RSS订阅源链接，如：https://feeds.bbci.co.uk/news/rss.xml"
              style="width: 100%; padding: 8px; border: 1px solid #e0e0e0; border-radius: 6px; font-size: 14px;"
              @blur="validateRssUrl"
            >
          </div>
          <div style="margin-bottom: 16px;">
            <label style="display: block; margin-bottom: 8px; font-weight: 500;">分类（可选）</label>
            <select 
              v-model="rssCategory"
              style="width: 100%; padding: 8px; border: 1px solid #e0e0e0; border-radius: 6px; font-size: 14px;"
            >
              <option value="">不分类</option>
              <option value="科技">科技</option>
              <option value="财经">财经</option>
              <option value="新闻">新闻</option>
              <option value="娱乐">娱乐</option>
              <option value="体育">体育</option>
              <option value="教育">教育</option>
              <option value="生活">生活</option>
              <option value="其他">其他</option>
            </select>
          </div>
          <div style="font-size: 12px; color: #666; margin-bottom: 16px;">
            💡 提示：输入RSS订阅源链接，系统会自动验证并获取订阅源信息
          </div>
          <div v-if="rssValidationStatus" style="margin-bottom: 16px;">
            <div 
              :class="['validation-status', rssValidationStatus.valid ? 'valid' : 'invalid']"
              style="padding: 8px 12px; border-radius: 6px; font-size: 13px;"
            >
              {{ rssValidationStatus.message }}
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer" style="padding: 16px 20px; border-top: 1px solid #e0e0e0; display: flex; justify-content: flex-end; gap: 8px;">
        <button class="btn btn-secondary" @click="showAddModal = false">取消</button>
        <button 
          class="btn btn-primary" 
          @click="confirmAddFeed" 
          :disabled="!canAddFeed"
        >
          确定
        </button>
      </div>
    </div>
  </div>

  <!-- 删除确认弹窗 -->
  <div v-if="showDeleteModal" class="modal-overlay" @click="cancelDelete">
    <div class="modal delete-modal" @click.stop>
      <div class="modal-header">
        <div class="modal-title">🗑️ 删除订阅源</div>
        <button class="modal-close" @click="cancelDelete">×</button>
      </div>
      <div class="modal-body">
        <div class="delete-warning">
          <div class="warning-icon">⚠️</div>
          <div class="warning-content">
            <h4>确定要删除订阅源 "{{ deleteTargetName }}" 吗？</h4>
            <p>删除后将同时删除该订阅源的所有文章，此操作不可恢复！</p>
            <div class="warning-details">
              <div class="detail-item">
                <span class="detail-label">订阅源名称：</span>
                <span class="detail-value">{{ deleteTargetName }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">文章数量：</span>
                <span class="detail-value">{{ feedArticleCounts[deleteTargetId] || 0 }} 篇</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">订阅源类型：</span>
                <span class="detail-value">{{ deleteTargetType === 'rss' ? 'RSS订阅源' : '微信公众号' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer" style="padding: 16px 20px; border-top: 1px solid #e0e0e0; display: flex; justify-content: flex-end; gap: 8px;">
        <button class="btn btn-secondary" @click="cancelDelete">取消</button>
        <button 
          class="btn btn-danger" 
          @click="confirmDelete"
          :disabled="deleting"
        >
          <span v-if="deleting" class="delete-spinner"></span>
          {{ deleting ? '删除中...' : '确认删除' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { listen } from '@tauri-apps/api/event'
import type { WeChatFeed, WeChatArticle, WeChatAccount, AllSettings } from '../types'
import {
  formatUtcTimestamp,
  formatIsoString,
  getBeijingDateRange,
  isUtcTimestampInBeijingRange
} from '../utils/timeUtils'

// RSS订阅源类型
interface RSSFeed {
  id: string
  title: string
  url: string
  website_url: string
  description?: string
  category?: string
  status: number
  last_fetched: number
  created_at: string
  updated_at: string
}

// 实时日志接口
interface RefreshLogEvent {
  timestamp: string
  level: string
  message: string
  feed_name?: string
}

interface RefreshProgressEvent {
  current: number
  total: number
  status: string
  log: string
  feed_name?: string
}

const feeds = ref<WeChatFeed[]>([])
const rssFeeds = ref<RSSFeed[]>([])
const articles = ref<any[]>([])
const selectedFeedId = ref('all-wechat')
const selectedArticles = ref<Set<string>>(new Set())
const loading = ref(false)
const loadingMore = ref(false)
const showAddModal = ref(false)
const wxsLink = ref('')
const dateFilter = ref('all')

// RSS相关状态
const feedType = ref<'wechat' | 'rss'>('wechat')
const rssUrl = ref('')
const rssCategory = ref('')
const rssValidationStatus = ref<{ valid: boolean; message: string } | null>(null)
const rssValidating = ref(false)

// 刷新进度相关
const showProgressModal = ref(false)
const refreshProgress = ref(0)
const refreshStatus = ref('')
const refreshTotal = ref(0)
const refreshCurrent = ref(0)
const refreshing = ref(false)
const refreshEta = ref('') // 预计剩余时间

// 实时日志相关
const refreshLogs = ref<RefreshLogEvent[]>([])
const maxLogLines = 100 // 最多保留100行日志

// 删除确认弹窗相关
const showDeleteModal = ref(false)
const deleteTargetId = ref('')
const deleteTargetName = ref('')
const deleteTargetType = ref<'wechat' | 'rss'>('wechat')
const deleting = ref(false)

const currentFeedName = computed(() => {
  if (selectedFeedId.value === 'all') {
    return '全部文章'
  }
  
  // 先查找微信公众号
  const feed = feeds.value.find(f => f.id === selectedFeedId.value)
  if (feed) {
    return feed.mp_name
  }
  
  // 再查找RSS订阅源
  const rssFeed = rssFeeds.value.find(f => f.id === selectedFeedId.value)
  if (rssFeed) {
    return rssFeed.title
  }
  
  return '未知订阅源'
})

const totalArticles = computed(() => {
  return articles.value.length
})

// 计算公众号文章总数
const totalWeChatArticles = computed(() => {
  return feeds.value.reduce((total, feed) => {
    return total + (feedArticleCounts.value[feed.id] || 0)
  }, 0)
})

// 计算RSS文章总数
const totalRssArticles = computed(() => {
  return rssFeeds.value.reduce((total, feed) => {
    return total + (feedArticleCounts.value[feed.id] || 0)
  }, 0)
})

const feedArticleCounts = ref<Record<string, number>>({})

// 获取订阅源的文章数量
const getFeedArticleCount = async (feedId: string) => {
  try {
    const feedArticles = await invoke<WeChatArticle[]>('get_feed_articles', { 
      feedId: feedId,
      limit: 1000 // 获取更多文章以准确计数
    })
    return feedArticles.length
  } catch (error) {
    console.error(`获取订阅源 ${feedId} 文章数量失败:`, error)
    return 0
  }
}

// 更新所有订阅源的文章数量
const updateFeedArticleCounts = async () => {
  const counts: Record<string, number> = {}
  
  // 更新微信公众号的文章数量
  for (const feed of feeds.value) {
    counts[feed.id] = await getFeedArticleCount(feed.id)
  }
  
  // 更新RSS订阅源的文章数量
  for (const rssFeed of rssFeeds.value) {
    counts[rssFeed.id] = await getFeedArticleCount(rssFeed.id)
  }
  
  feedArticleCounts.value = counts
}

const filteredArticles = computed(() => {
  if (dateFilter.value === 'all') {
    return articles.value
  }
  
  // 使用统一的时间处理工具函数
  return articles.value.filter(article => {
    return isUtcTimestampInBeijingRange(article.publish_time, dateFilter.value as any)
  })
})

const allSelected = computed(() => {
  return filteredArticles.value.length > 0 && 
         filteredArticles.value.every(article => selectedArticles.value.has(article.id))
})

const hasMore = ref(false)

// RSS相关计算属性
const canAddFeed = computed(() => {
  if (feedType.value === 'wechat') {
    return wxsLink.value.trim().length > 0
  } else {
    return rssUrl.value.trim().length > 0 && 
           rssValidationStatus.value?.valid === true && 
           !rssValidating.value
  }
})

// 事件监听器
let progressUnlisten: (() => void) | null = null
let logUnlisten: (() => void) | null = null

onMounted(async () => {
  await loadFeeds()
  await setupEventListeners()
  // 默认选择"全部文章"并加载文章
  await selectFeed('all')
})

onUnmounted(() => {
  // 清理事件监听器
  if (progressUnlisten) {
    progressUnlisten()
  }
  if (logUnlisten) {
    logUnlisten()
  }
})

// 设置事件监听器
const setupEventListeners = async () => {
  try {
    // 监听进度事件
    progressUnlisten = await listen<RefreshProgressEvent>('refresh-progress', (event) => {
      const data = event.payload
      console.log('收到进度事件:', data)

      // 更新进度状态
      refreshProgress.value = data.total > 0 ? (data.current / data.total) * 100 : 0
      refreshStatus.value = data.status
      refreshCurrent.value = data.current
      refreshTotal.value = data.total

      // 计算预计剩余时间 (平均每个订阅源2-4秒)
      if (data.current > 0 && data.total > data.current) {
        const remaining = data.total - data.current
        const avgTimePerSource = 3 // 平均3秒每个订阅源
        const estimatedSeconds = remaining * avgTimePerSource

        if (estimatedSeconds < 60) {
          refreshEta.value = `${estimatedSeconds}秒`
        } else {
          refreshEta.value = `${Math.ceil(estimatedSeconds / 60)}分钟`
        }
      } else if (data.current === data.total) {
        refreshEta.value = '即将完成'
      } else {
        refreshEta.value = '计算中...'
      }

      // 确保进度弹窗显示
      if (!showProgressModal.value) {
        showProgressModal.value = true
      }
    })

    // 监听中断事件
    const interruptUnlisten = await listen<RefreshProgressEvent>('refresh-interrupted', (event) => {
      const data = event.payload
      console.log('收到中断事件:', data)

      refreshStatus.value = data.status
      refreshing.value = false

      // 2秒后自动关闭弹窗
      setTimeout(() => {
        showProgressModal.value = false
        refreshEta.value = ''
      }, 2000)
    })

    // 监听日志事件
    logUnlisten = await listen<RefreshLogEvent>('refresh-log', (event) => {
      const logEntry = event.payload
      console.log('收到日志事件:', logEntry)

      // 添加到日志列表
      refreshLogs.value.push(logEntry)

      // 限制日志行数
      if (refreshLogs.value.length > maxLogLines) {
        refreshLogs.value = refreshLogs.value.slice(-maxLogLines)
      }

      // 自动滚动到底部
      nextTick(() => {
        const logContainer = document.querySelector('.refresh-logs-container')
        if (logContainer) {
          logContainer.scrollTop = logContainer.scrollHeight
        }
      })
    })

    console.log('事件监听器设置完成')
  } catch (error) {
    console.error('设置事件监听器失败:', error)
  }
}

// 清空日志
const clearLogs = () => {
  refreshLogs.value = []
}

const loadFeeds = async () => {
  try {
    // 同时获取微信公众号和RSS订阅源
    feeds.value = await invoke<WeChatFeed[]>('get_all_feeds')
    rssFeeds.value = await invoke<RSSFeed[]>('get_all_rss_feeds')
    // 加载完订阅源后，更新文章数量
    await updateFeedArticleCounts()
  } catch (error) {
    console.error('加载订阅源失败:', error)
  }
}

const selectFeed = async (feedId: string) => {
  selectedFeedId.value = feedId
  selectedArticles.value.clear()
  
  if (feedId === 'all') {
    await loadAllArticles()
  } else if (feedId === 'all-wechat') {
    await loadWeChatArticles()
  } else if (feedId === 'all-rss') {
    await loadRssArticles()
  } else {
    await loadFeedArticles(feedId)
  }
}

const loadAllArticles = async () => {
  loading.value = true
  try {
    articles.value = await invoke<WeChatArticle[]>('get_all_articles')
  } catch (error) {
    console.error('加载文章失败:', error)
  } finally {
    loading.value = false
  }
}

const loadFeedArticles = async (feedId: string) => {
  loading.value = true
  try {
    articles.value = await invoke<WeChatArticle[]>('get_feed_articles', { feedId: feedId })
  } catch (error) {
    console.error('加载文章失败:', error)
    articles.value = [] // 确保出错时清空文章列表
  } finally {
    loading.value = false
  }
}

const loadWeChatArticles = async () => {
  loading.value = true
  try {
    // 获取所有文章（统一格式）
    const allArticles = await invoke<any[]>('get_all_articles')
    // 过滤出微信公众号文章（source_type为'WeChat'）
    articles.value = allArticles.filter(article => article.source_type === 'WeChat')
  } catch (error) {
    console.error('加载微信公众号文章失败:', error)
    articles.value = []
  } finally {
    loading.value = false
  }
}

const loadRssArticles = async () => {
  loading.value = true
  try {
    // 获取所有文章（统一格式）
    console.log('正在获取所有文章（统一格式）...')
    const allArticles = await invoke<any[]>('get_all_articles')
    console.log('所有文章:', allArticles)
    
    // 过滤出RSS文章（source_type为'RSS'）
    const rssArticles = allArticles.filter(article => article.source_type === 'RSS')
    console.log('过滤后的RSS文章:', rssArticles)
    
    articles.value = rssArticles
    console.log('articles.value.length:', articles.value.length)
    console.log('第一篇文章:', articles.value[0])
    
    // 如果没有RSS文章，尝试调试
    if (rssArticles.length === 0) {
      console.log('没有找到RSS文章，尝试调试...')
      try {
        const debugInfo = await invoke<string>('debug_articles', { limit: 10 })
        console.log('调试信息:', debugInfo)
      } catch (debugError) {
        console.error('调试失败:', debugError)
      }
    }
  } catch (error) {
    console.error('加载RSS文章失败:', error)
    articles.value = []
  } finally {
    loading.value = false
  }
}

const refreshCurrentFeed = async () => {
  if (selectedFeedId.value === 'all') {
    // 刷新所有订阅源
    await refreshAllFeeds()
  } else {
    // 刷新单个订阅源
    await refreshSingleFeed(selectedFeedId.value)
  }
}

// 中断刷新任务
const interruptRefresh = async () => {
  try {
    console.log('请求中断刷新任务')
    const result = await invoke<string>('interrupt_refresh_refresh')
    console.log('中断请求结果:', result)

    // 更新UI状态
    refreshStatus.value = '正在中断任务...'

  } catch (error) {
    console.error('中断刷新失败:', error)
    alert('中断失败: ' + error)
  }
}

// 关闭进度弹窗
const closeProgressModal = () => {
  if (refreshStatus.value.includes('中断')) {
    // 如果是中断状态，重置状态
    refreshing.value = false
    refreshEta.value = ''
  }
  showProgressModal.value = false
}

const refreshAllFeeds = async () => {
  if (feeds.value.length === 0) {
    alert('暂无订阅源可刷新')
    return
  }

  // 设置刷新状态
  refreshing.value = true

  try {
    console.log('开始批量刷新所有订阅源，共', feeds.value.length, '个')

    // 显示进度弹窗
    showProgressModal.value = true
    refreshStatus.value = '正在准备刷新订阅源...'
    refreshProgress.value = 0
    refreshTotal.value = feeds.value.length
    refreshCurrent.value = 0

    // 延迟显示进度弹窗，确保UI更新
    await new Promise(resolve => setTimeout(resolve, 100))

    refreshStatus.value = `正在刷新 0 / ${feeds.value.length} 个订阅源...`

    // 使用新的批量刷新命令
    const result = await invoke<string>('refresh_all_feeds')
    console.log('批量刷新结果:', result)

    // 更新进度
    refreshProgress.value = 80
    refreshStatus.value = '正在更新界面...'
    refreshCurrent.value = feeds.value.length

    // 重新加载订阅源列表（更新同步时间）
    await loadFeeds()

    // 重新加载文章列表
    await loadAllArticles()

    refreshProgress.value = 100
    refreshStatus.value = '批量刷新完成！'

    // 延迟关闭进度弹窗
    setTimeout(() => {
      showProgressModal.value = false
    }, 2000)

    // 显示结果
    console.log('批量刷新成功:', result)

    // 使用浏览器通知
    if ('Notification' in window) {
      new Notification('批量刷新完成', {
        body: result,
        icon: '/favicon.ico'
      })
    }
  } catch (error) {
    console.error('批量刷新失败:', error)
    const errorMessage = error instanceof Error ? error.message : String(error)

    refreshStatus.value = '批量刷新失败'
    refreshProgress.value = 0

    // 对于账号不可用错误，不自动关闭弹窗
    if (!(errorMessage.includes('所有账号都不可用') ||
          errorMessage.includes('账号被封禁') ||
          errorMessage.includes('黑名单') ||
          errorMessage.includes('WeReadError400'))) {
      // 延迟关闭进度弹窗
      setTimeout(() => {
        showProgressModal.value = false
      }, 3000)
    }

    // 检查是否是账号黑名单相关错误
    if (errorMessage.includes('所有账号都不可用') ||
        errorMessage.includes('账号被封禁') ||
        errorMessage.includes('黑名单') ||
        errorMessage.includes('WeReadError400')) {
      // 显示友好的账号状态提示
      const userFriendlyMessage = `⚠️ 账号暂时不可用

检测到账号可能被微信临时限制访问，这是正常现象。

📱 可能原因：
• 账号请求过于频繁触发保护机制
• 微信对第三方接口访问限制
• 账号正在冷却期

🔧 解决方案：
• 等待24小时后自动解除限制
• 添加更多微信账号作为备用
• 减少刷新频率，避免连续操作

💡 小贴士：
• 建议添加2-3个微信账号轮换使用
• 每个账号每天刷新1-2次为佳
• 账号会在24小时后自动恢复可用

详细错误：${errorMessage}`

      // 保持弹窗不自动关闭，让用户看到错误信息
      refreshStatus.value = '❌ 所有账号都不可用'
      refreshProgress.value = 0
      
      // 添加错误日志到日志列表
      const errorLogEntry: RefreshLogEvent = {
        timestamp: new Date().toISOString(),
        level: 'error',
        message: '所有账号都不可用，已中止刷新操作',
        feed_name: null
      }
      refreshLogs.value.push(errorLogEntry)
      
      // 显示详细错误信息
      alert(userFriendlyMessage)
      
      // 不自动关闭弹窗，让用户手动关闭
      return
    } else {
      alert(`批量刷新失败: ${errorMessage}`)
    }
  } finally {
    refreshing.value = false
  }
}

const refreshSingleFeed = async (feedId: string) => {
  // 设置刷新状态
  refreshing.value = true

  // 显示进度弹窗
  showProgressModal.value = true
  refreshStatus.value = '正在初始化...'
  refreshProgress.value = 0
  refreshTotal.value = 1
  refreshCurrent.value = 0

  try {
    console.log('开始刷新订阅源:', feedId)

    // 获取订阅源信息用于显示
    const feed = feeds.value.find(f => f.id === feedId)
    const feedName = feed?.mp_name || '未知订阅源'

    refreshStatus.value = `正在刷新 ${feedName}...`
    refreshProgress.value = 10

    const newCount = await invoke<number>('refresh_feed', { feedId: feedId })
    console.log(`刷新完成，新增 ${newCount} 篇文章`)

    refreshProgress.value = 80
    refreshStatus.value = '正在更新界面...'

    // 重新加载订阅源列表（更新同步时间）
    await loadFeeds()

    // 重新加载当前订阅源的文章列表
    await loadFeedArticles(feedId)

    refreshProgress.value = 100
    refreshStatus.value = '刷新完成！'

    // 延迟关闭进度弹窗
    setTimeout(() => {
      showProgressModal.value = false
    }, 1500)

    // 显示成功消息
    const statusMessage = `刷新完成，新增 ${newCount} 篇文章`
    console.log(statusMessage)

    // 使用浏览器通知
    if (newCount > 0 && 'Notification' in window) {
      new Notification('刷新成功', {
        body: statusMessage,
        icon: '/favicon.ico'
      })
    }
  } catch (error) {
    console.error('刷新失败:', error)
    const errorMessage = error instanceof Error ? error.message : String(error)

    refreshStatus.value = '刷新失败'
    refreshProgress.value = 0

    // 对于账号不可用错误，不自动关闭弹窗
    if (!(errorMessage.includes('所有账号都不可用') ||
          errorMessage.includes('账号被封禁') ||
          errorMessage.includes('黑名单') ||
          errorMessage.includes('WeReadError400'))) {
      // 延迟关闭进度弹窗
      setTimeout(() => {
        showProgressModal.value = false
      }, 2000)
    }

    // 检查是否是账号黑名单相关错误
    if (errorMessage.includes('所有账号都不可用') ||
        errorMessage.includes('账号被封禁') ||
        errorMessage.includes('黑名单') ||
        errorMessage.includes('WeReadError400')) {
      // 显示友好的账号状态提示
      const userFriendlyMessage = `⚠️ 账号暂时不可用

检测到账号可能被微信临时限制访问，这是正常现象。

📱 可能原因：
• 账号请求过于频繁触发保护机制
• 微信对第三方接口访问限制
• 账号正在冷却期

🔧 解决方案：
• 等待24小时后自动解除限制
• 添加更多微信账号作为备用
• 减少刷新频率，避免连续操作

💡 小贴士：
• 建议添加2-3个微信账号轮换使用
• 每个账号每天刷新1-2次为佳
• 账号会在24小时后自动恢复可用

详细错误：${errorMessage}`

      // 保持弹窗不自动关闭，让用户看到错误信息
      refreshStatus.value = '❌ 所有账号都不可用'
      refreshProgress.value = 0
      
      // 添加错误日志到日志列表
      const errorLogEntry: RefreshLogEvent = {
        timestamp: new Date().toISOString(),
        level: 'error',
        message: '所有账号都不可用，已中止刷新操作',
        feed_name: feedName
      }
      refreshLogs.value.push(errorLogEntry)
      
      // 显示详细错误信息
      alert(userFriendlyMessage)
      
      // 不自动关闭弹窗，让用户手动关闭
      return
    } else {
      alert(`刷新失败: ${errorMessage}`)
    }
  } finally {
    refreshing.value = false
  }
}

const toggleSelectAll = () => {
  if (allSelected.value) {
    // 取消全选
    filteredArticles.value.forEach(article => {
      selectedArticles.value.delete(article.id)
    })
  } else {
    // 全选
    filteredArticles.value.forEach(article => {
      selectedArticles.value.add(article.id)
    })
  }
}

const toggleArticleSelection = (articleId: string) => {
  if (selectedArticles.value.has(articleId)) {
    selectedArticles.value.delete(articleId)
  } else {
    selectedArticles.value.add(articleId)
  }
}

const clearSelection = () => {
  selectedArticles.value.clear()
}

const startAnalysis = async () => {
  const articleIds = Array.from(selectedArticles.value)
  if (articleIds.length === 0) {
    alert('请先选择要分析的文章')
    return
  }

  try {
    // 获取默认提示词模板
    const defaultTemplate = await invoke<any>('get_default_prompt_template')
    if (!defaultTemplate) {
      alert('请先在设置页面配置提示词模板')
      return
    }
    
    console.log('开始分析文章，文章数量:', articleIds.length)

    // 调用分析命令（使用新的参数格式）
    const taskId: string = await invoke('start_analysis', {
      articleIds: articleIds
    })
    
    alert(`分析任务已启动，任务ID: ${taskId.slice(0, 8)}...`)
    
    // 清空选择
    selectedArticles.value.clear()
    
    // 可选：自动跳转到分析结果页面
    // 这里可以添加跳转逻辑或通知用户查看结果
    
  } catch (error) {
    console.error('启动分析失败:', error)
    alert(`启动分析失败: ${error}`)
  }
}

const showAddFeedModal = () => {
  showAddModal.value = true
  wxsLink.value = ''
  rssUrl.value = ''
  rssCategory.value = ''
  rssValidationStatus.value = null
  feedType.value = 'wechat'
}

// RSS验证函数
const validateRssUrl = async () => {
  if (!rssUrl.value.trim()) {
    rssValidationStatus.value = null
    return
  }

  rssValidating.value = true
  rssValidationStatus.value = {
    valid: false,
    message: '正在验证RSS链接...'
  }

  try {
    const isValid = await invoke<boolean>('validate_rss_url', { 
      url: rssUrl.value.trim() 
    })
    
    if (isValid) {
      rssValidationStatus.value = {
        valid: true,
        message: '✅ RSS链接验证成功'
      }
    } else {
      rssValidationStatus.value = {
        valid: false,
        message: '❌ RSS链接无效，请检查链接是否正确'
      }
    }
  } catch (error) {
    console.error('RSS验证失败:', error)
    rssValidationStatus.value = {
      valid: false,
      message: `❌ 验证失败: ${error}`
    }
  } finally {
    rssValidating.value = false
  }
}

const confirmAddFeed = async () => {
  try {
    if (feedType.value === 'wechat') {
      // 添加公众号订阅源
      if (!wxsLink.value.trim()) {
        return
      }
      
      console.log('开始添加公众号订阅源...')
      
      // 获取第一个可用账号
      const accounts = await invoke<WeChatAccount[]>('get_all_accounts')
      console.log('获取到账号列表:', accounts.length)
      
      const availableAccount = accounts.find((acc: WeChatAccount) => acc.status === 1)
      
      if (!availableAccount) {
        alert('请先添加并启用一个微信账号')
        return
      }
      
      console.log('使用账号:', availableAccount.name)
      
      // 解析链接，每行一个
      const links = wxsLink.value.trim().split('\n').filter(link => link.trim())
      console.log('要处理的链接数量:', links.length)
      
      let successCount = 0
      let failCount = 0
      
      for (const link of links) {
        try {
          console.log('处理链接:', link.trim())
          const feedName = await invoke('add_feed_from_url', {
            url: link.trim(),
            accountId: availableAccount.id
          })
          console.log('添加订阅源成功:', feedName)
          successCount++
          
          // 添加延迟避免快速连续调用
          await new Promise(resolve => setTimeout(resolve, 200))
        } catch (error) {
          console.error('添加订阅源失败:', link, error)
          failCount++
        }
      }
      
      // 显示结果
      if (successCount > 0) {
        alert(`成功添加 ${successCount} 个公众号订阅源${failCount > 0 ? `，${failCount} 个失败` : ''}`)
      } else {
        alert('添加公众号订阅源失败，请检查链接是否正确')
      }
    } else {
      // 添加RSS订阅源
      if (!rssUrl.value.trim()) {
        return
      }
      
      console.log('开始添加RSS订阅源...')
      
      const feedName = await invoke('add_rss_feed', {
        url: rssUrl.value.trim(),
        category: rssCategory.value || null
      })
      
      console.log('添加RSS订阅源成功:', feedName)
      alert(`成功添加RSS订阅源: ${feedName}`)
    }
    
    showAddModal.value = false
    wxsLink.value = ''
    rssUrl.value = ''
    rssCategory.value = ''
    rssValidationStatus.value = null
    
    // 重新加载订阅源列表
    await loadFeeds()
    
  } catch (error) {
    console.error('添加订阅源失败:', error)
    alert(`添加订阅源失败: ${error}`)
  }
}

const filterByDate = () => {
  // 日期筛选逻辑已在 computed 中实现
}

const loadMoreArticles = () => {
  // 实现加载更多文章的逻辑
  console.log('加载更多文章')
}

const formatLastUpdated = (lastUpdated: string | null) => {
  if (!lastUpdated) {
    return '从未更新'
  }
  
  // 使用统一的时间处理工具函数
  const relativeTime = formatIsoString(lastUpdated, 'relative')
  return `${relativeTime}更新`
}

const formatTime = (timestamp: number) => {
  // 使用统一的时间处理工具函数
  return formatUtcTimestamp(timestamp, 'datetime')
}

const formatLogTime = (timestamp: string) => {
  // 使用统一的时间处理工具函数
  return formatIsoString(timestamp, 'time')
}

const openArticle = async (article: any) => {
  console.log('=== 订阅源页面点击文章 ===')
  console.log('文章对象:', article)
  console.log('文章ID:', article.id)
  console.log('文章标题:', article.title)
  console.log('文章URL:', article.url)
  console.log('文章类型:', article.source_type)
  
  // 优先使用文章中的URL，如果没有则根据类型构造
  let url = article.url
  if (!url || url.trim() === '') {
    if (article.source_type === 'RSS') {
      // RSS文章通常有完整的URL，如果为空则无法构造
      console.error('RSS文章URL为空，无法打开')
      alert('RSS文章链接无效')
      return
    } else {
      // 微信文章可以构造URL
      url = `https://mp.weixin.qq.com/s/${article.id}`
      console.log('URL为空，构造微信文章URL:', url)
    }
  } else {
    console.log('使用原始URL:', url)
  }
  
  // 方法1：使用 Tauri 的 shell API 打开链接（推荐）
  try {
    await open(url)
    console.log('使用 Tauri shell API 成功打开链接')
    return
  } catch (error) {
    console.error('Tauri shell API 失败:', error)
  }
  
  // 方法2：尝试直接使用 window.open（备用）
  try {
    const newWindow = window.open(url, '_blank', 'noopener,noreferrer')
    console.log('window.open 结果:', newWindow)
    if (newWindow) {
      console.log('链接成功打开')
      return
    }
  } catch (error) {
    console.error('window.open 失败:', error)
  }
  
  // 方法3：创建临时链接并模拟点击（备用）
  try {
    const link = document.createElement('a')
    link.href = url
    link.target = '_blank'
    link.rel = 'noopener noreferrer'
    link.style.display = 'none'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    console.log('使用临时链接方法打开')
    return
  } catch (error) {
    console.error('临时链接方法失败:', error)
  }
  
  // 方法4：最后备用方案 - 复制到剪贴板
  console.log('所有方法都失败，复制到剪贴板')
  try {
    await navigator.clipboard.writeText(url)
    console.log('链接已复制到剪贴板')
    alert('链接已复制到剪贴板: ' + url)
  } catch (err) {
    console.error('复制到剪贴板也失败:', err)
    alert('无法打开链接，请手动复制: ' + url)
  }
}


// 删除确认弹窗相关函数
const cancelDelete = () => {
  showDeleteModal.value = false
  deleteTargetId.value = ''
  deleteTargetName.value = ''
  deleteTargetType.value = 'wechat'
  deleting.value = false
}

const confirmDelete = async () => {
  if (deleting.value) {
    return
  }

  deleting.value = true

  try {
    console.log('开始删除订阅源:', deleteTargetId.value, deleteTargetName.value)
    
    // 根据类型调用不同的删除命令
    if (deleteTargetType.value === 'rss') {
      await invoke('delete_rss_feed', { feedId: deleteTargetId.value })
      console.log('RSS订阅源删除成功:', deleteTargetName.value)
    } else {
      await invoke('delete_feed', { feedId: deleteTargetId.value })
      console.log('订阅源删除成功:', deleteTargetName.value)
    }
    
    // 重新加载订阅源列表
    await loadFeeds()
    
    // 如果当前选中的是被删除的订阅源，切换到"全部文章"
    if (selectedFeedId.value === deleteTargetId.value) {
      await selectFeed('all')
    }
    
    // 重新加载文章列表（如果当前在"全部文章"视图）
    if (selectedFeedId.value === 'all') {
      await loadAllArticles()
    }
    
    // 关闭弹窗
    showDeleteModal.value = false
    
    // 显示成功消息
    const feedTypeText = deleteTargetType.value === 'rss' ? 'RSS订阅源' : '订阅源'
    alert(`${feedTypeText} "${deleteTargetName.value}" 及其所有文章已成功删除`)
    
  } catch (error) {
    console.error('删除订阅源失败:', error)
    alert(`删除订阅源失败: ${error}`)
  } finally {
    deleting.value = false
  }
}

// 删除微信公众号订阅源
const deleteFeed = (feedId: string, feedName: string) => {
  deleteTargetId.value = feedId
  deleteTargetName.value = feedName
  deleteTargetType.value = 'wechat'
  showDeleteModal.value = true
}

// 删除RSS订阅源
const deleteRssFeed = (feedId: string, feedName: string) => {
  deleteTargetId.value = feedId
  deleteTargetName.value = feedName
  deleteTargetType.value = 'rss'
  showDeleteModal.value = true
}
</script>

<style scoped>
.feeds-layout {
  display: flex;
  height: calc(100vh - 140px);
  background: #f8f9fa;
}

/* 左侧订阅源列表 */
.feeds-sidebar {
  width: 256px;
  background: white;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
}

.feeds-sidebar-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.feeds-sidebar-header h3 {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: #333;
}

.feeds-actions {
  display: flex;
  gap: 8px;
}

.feeds-sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.feed-list {
  display: flex;
  flex-direction: column;
}

.feed-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
  position: relative;
}

.feed-item:hover {
  background: #f8f9fa;
}

.feed-item.active {
  background: rgba(59, 130, 246, 0.08);
  border-color: transparent;
  color: #3b82f6;
}

.feed-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: #3b82f6;
}

.feed-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #f0f0f0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  color: #666;
  margin-right: 12px;
  flex-shrink: 0;
  overflow: hidden;
}

.feed-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.feed-item.active .feed-avatar {
  background: #3b82f6;
  color: white;
}

.feed-info {
  flex: 1;
  min-width: 0;
}

.feed-info h4 {
  font-size: 14px;
  font-weight: 500;
  margin: 0 0 2px 0;
  color: inherit;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.feed-meta {
  font-size: 12px;
  color: #999;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.feed-item.active .feed-meta {
  color: #3b82f6;
}

.feed-category {
  color: #666;
  font-weight: 500;
}

/* 订阅源内容区域 */
.feed-content {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
}

/* 订阅源操作按钮 */
.feed-actions {
  display: flex;
  align-items: center;
  margin-left: 8px;
}

.btn-delete {
  background: none;
  border: none;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  color: #999;
  transition: all 0.2s ease;
  opacity: 0;
  transform: scale(0.9);
}

.feed-item:hover .btn-delete {
  opacity: 1;
  transform: scale(1);
}

.btn-delete:hover {
  background: #ff4444;
  color: white;
  transform: scale(1.1);
}

.btn-delete:active {
  transform: scale(0.95);
}

/* 右侧文章面板 */
.articles-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: white;
}

.articles-header {
  padding: 16px 24px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
}

.articles-header h3 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #333;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
}

.articles-actions {
  display: flex;
  gap: 16px;
  align-items: center;
}

.date-filter select {
  padding: 6px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  font-size: 14px;
  background: white;
  cursor: pointer;
}

.articles-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.articles-table {
  width: 100%;
  border-collapse: collapse;
  background: white;
}

.articles-table th {
  background: #f8f9fa;
  padding: 12px 16px;
  text-align: left;
  font-weight: 600;
  color: #333;
  border-bottom: 1px solid #e0e0e0;
  font-size: 14px;
}

.articles-table td {
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  vertical-align: top;
}

.articles-table tr:hover {
  background: #f8f9fa;
}

.articles-table tr.selected {
  background: rgba(59, 130, 246, 0.05);
}

.articles-table tr.selected td {
  border-bottom-color: rgba(59, 130, 246, 0.1);
}

.article-title-cell {
  width: 100%;
}

.article-title-link {
  color: #333;
  text-decoration: none;
  font-size: 14px;
  line-height: 1.4;
  display: block;
}

.article-title-link:hover {
  color: #3b82f6;
}

.article-title-link:visited {
  color: #999;
}

.article-time-cell {
  width: 180px;
  font-size: 13px;
  color: #666;
  white-space: nowrap;
}

.load-more-container {
  display: flex;
  justify-content: center;
  padding: 16px;
}

.btn-load-more {
  padding: 8px 16px;
  background: #f8f9fa;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  color: #333;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.btn-load-more:hover {
  background: #e9ecef;
}

.btn-load-more:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.articles-empty {
  text-align: center;
  padding: 60px 20px;
  color: #666;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 16px;
  margin-bottom: 8px;
}

.empty-description {
  font-size: 14px;
  color: #999;
}

.articles-analysis-controls {
  padding: 16px 24px;
  border-top: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
}

.selected-info {
  display: flex;
  align-items: center;
  font-size: 14px;
  color: #666;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.btn-primary {
  background: #3498db;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2980b9;
}

.btn-primary:disabled {
  background: #bdc3c7;
  cursor: not-allowed;
}

.btn-secondary {
  background: #ecf0f1;
  color: #2c3e50;
}

.btn-secondary:hover {
  background: #bdc3c7;
}

.btn-small {
  padding: 6px 12px;
  font-size: 12px;
}

/* 模态框 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 12px;
  width: 400px;
  max-width: 90%;
  box-shadow: 0 10px 30px rgba(0,0,0,0.3);
}

.modal-header {
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
}

.modal-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
}

.modal-body {
  padding: 24px;
}

/* 进度弹窗样式 */
.progress-modal {
  width: 600px;
  max-height: 80vh;
}

/* 实时日志样式 */
.refresh-logs-section {
  margin: 20px 0;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
}

.logs-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.refresh-logs-container {
  max-height: 200px;
  overflow-y: auto;
  background: #fafafa;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
}

.log-entry {
  display: flex;
  align-items: flex-start;
  padding: 6px 12px;
  border-bottom: 1px solid #f0f0f0;
  gap: 8px;
}

.log-entry:last-child {
  border-bottom: none;
}

.log-entry:hover {
  background: #f0f0f0;
}

.log-time {
  color: #666;
  white-space: nowrap;
  font-size: 11px;
  min-width: 80px;
}

.log-level {
  white-space: nowrap;
  font-weight: 600;
  min-width: 50px;
  font-size: 10px;
}

.log-message {
  flex: 1;
  word-break: break-word;
}

.log-feed-name {
  color: #3b82f6;
  font-weight: 600;
  margin-right: 4px;
}

/* 日志级别颜色 */
.log-info .log-level {
  color: #10b981;
}

.log-warn .log-level {
  color: #f59e0b;
}

.log-error .log-level {
  color: #ef4444;
}

.log-info {
  border-left: 3px solid #10b981;
}

.log-warn {
  border-left: 3px solid #f59e0b;
}

.log-error {
  border-left: 3px solid #ef4444;
}

.log-empty {
  padding: 20px;
  text-align: center;
  color: #999;
  font-style: italic;
}

.progress-info {
  margin-bottom: 20px;
}

.progress-status {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin-bottom: 8px;
}

.progress-details {
  font-size: 14px;
  color: #666;
  display: flex;
  gap: 12px;
}

.progress-bar-container {
  width: 100%;
  height: 8px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 20px;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-tips {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  border-left: 4px solid #667eea;
}

.progress-tips div {
  font-size: 13px;
  color: #666;
  line-height: 1.4;
}

/* 刷新按钮动画 */
.refresh-spinner {
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

.btn-secondary:disabled {
  opacity: 0.7;
  cursor: not-allowed;
  position: relative;
}

.btn-secondary:disabled .refresh-spinner {
  animation: spin 1s linear infinite;
}

/* RSS相关样式 */
.add-feed-modal {
  width: 500px;
}

.feed-type-tabs {
  display: flex;
  margin-bottom: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.tab-button {
  flex: 1;
  padding: 12px 16px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: #666;
  border-bottom: 2px solid transparent;
  transition: all 0.2s ease;
}

.tab-button:hover {
  color: #333;
  background: #f8f9fa;
}

.tab-button.active {
  color: #3b82f6;
  border-bottom-color: #3b82f6;
  background: #f8f9fa;
}

.feed-type-content {
  margin-top: 20px;
}

.validation-status {
  border-radius: 6px;
  font-size: 13px;
  line-height: 1.4;
}

.validation-status.valid {
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.validation-status.invalid {
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

/* 删除确认弹窗样式 */
.delete-modal {
  width: 450px;
}

.delete-warning {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.warning-icon {
  font-size: 48px;
  color: #f59e0b;
  flex-shrink: 0;
  margin-top: 8px;
}

.warning-content {
  flex: 1;
}

.warning-content h4 {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin: 0 0 12px 0;
  line-height: 1.4;
}

.warning-content p {
  font-size: 14px;
  color: #666;
  margin: 0 0 16px 0;
  line-height: 1.5;
}

.warning-details {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  border-left: 4px solid #f59e0b;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.detail-item:last-child {
  margin-bottom: 0;
}

.detail-label {
  font-size: 13px;
  color: #666;
  font-weight: 500;
}

.detail-value {
  font-size: 13px;
  color: #333;
  font-weight: 600;
}

.btn-danger {
  background: #dc3545;
  color: white;
  border: none;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
}

.btn-danger:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.delete-spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-right: 6px;
}

/* 进度弹窗moda-actions样式 */
.modal-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: auto;
}

/* 预计剩余时间样式 */
.progress-eta {
  color: #667eea;
  font-weight: 500;
  background: rgba(102, 126, 234, 0.1);
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
}

/* 刷新日志区域样式 */
.refresh-logs-section {
  margin-top: 20px;
  border-top: 1px solid #e9ecef;
  padding-top: 16px;
}

.refresh-logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.refresh-logs-container {
  max-height: 200px;
  overflow-y: auto;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  padding: 8px;
}

.log-entry {
  display: flex;
  gap: 8px;
  padding: 6px 8px;
  font-size: 11px;
  border-bottom: 1px solid rgba(0,0,0,0.05);
  align-items: flex-start;
  font-family: monospace;
}

.log-entry:last-child {
  border-bottom: none;
}

.log-time {
  color: #6c757d;
  min-width: 70px;
  flex-shrink: 0;
}

.log-level {
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 9px;
  font-weight: 600;
  min-width: 40px;
  text-align: center;
  flex-shrink: 0;
}

.log-entry.log-info .log-level {
  background: #d1ecf1;
  color: #0c5460;
}

.log-entry.log-warn .log-level {
  background: #fff3cd;
  color: #856404;
}

.log-entry.log-error .log-level {
  background: #f8d7da;
  color: #721c24;
}

.log-message {
  flex: 1;
  color: #495057;
  word-break: break-word;
  line-height: 1.4;
}

.log-feed-name {
  color: #667eea;
  font-weight: 600;
}

.log-empty {
  text-align: center;
  color: #6c757d;
  padding: 40px;
  font-style: italic;
}

/* 响应式设计调整 */
@media (max-width: 768px) {
  .modal-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .modal-actions {
    justify-content: center;
  }

  .progress-details {
    flex-direction: column;
    gap: 4px;
    align-items: flex-start;
  }
}
</style>
