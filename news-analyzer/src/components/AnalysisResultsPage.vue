<template>
  <div class="analysis-results-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-text">
          <h1 class="page-title">📊 智能分析结果</h1>
          <p class="page-subtitle">查看AI分析的新闻结果和统计数据</p>
        </div>
        <div class="header-stats">
          <div class="stat-card">
            <div class="stat-number">{{ analyzedNews.length }}</div>
            <div class="stat-label">总结果</div>
          </div>
          <div class="stat-card">
            <div class="stat-number">{{ selectedNews.length }}</div>
            <div class="stat-label">已选中</div>
          </div>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary" @click="refreshResults">
          <span class="btn-icon">🔄</span>
          刷新结果
        </button>
        <button class="btn btn-secondary" @click="showExportModal = true" :disabled="filteredNews.length === 0">
          <span class="btn-icon">📤</span>
          导出结果
        </button>
        <button class="btn btn-primary" @click="toggleSelectAll" :disabled="filteredNews.length === 0">
          <span class="btn-icon">{{ allSelected ? '☑️' : '⬜' }}</span>
          {{ allSelected ? '取消全选筛选结果' : '全选筛选结果' }}
        </button>
        <button class="btn btn-danger" @click="deleteSelected" :disabled="selectedNews.length === 0">
          <span class="btn-icon">🗑️</span>
          删除选中 ({{ selectedNews.length }})
        </button>
        <button class="btn btn-secondary" @click="showLogsPanel = !showLogsPanel">
          <span class="btn-icon">📋</span>
          {{ showLogsPanel ? '隐藏日志' : '显示日志' }}
          <span v-if="logStats.error > 0" class="error-indicator">{{ logStats.error }}</span>
        </button>
      </div>
    </div>

    <!-- 筛选器 -->
    <div class="filters-section">
      <div class="filters-header">
        <h3 class="filters-title">🔍 筛选条件</h3>
        <div class="filter-status-text">{{ filterStatusText }}</div>
        <div class="filters-actions">
          <button class="btn btn-sm btn-secondary" @click="applyFilters">
            应用筛选
          </button>
          <button class="btn btn-sm btn-outline" @click="clearFilters">
            清除筛选
          </button>
        </div>
      </div>
      <div class="filter-row">
        <div class="filter-group">
          <label class="filter-label">行业类型</label>
          <select v-model="filters.industryType" class="filter-select">
            <option value="">全部行业</option>
            <option v-for="type in uniqueIndustryTypes" :key="type" :value="type">
              {{ type }}
            </option>
          </select>
        </div>

        <div class="filter-group">
          <label class="filter-label">新闻类型</label>
          <select v-model="filters.newsType" class="filter-select">
            <option value="">全部类型</option>
            <option v-for="type in uniqueNewsTypes" :key="type" :value="type">
              {{ type }}
            </option>
          </select>
        </div>

        <div class="filter-group">
          <label class="filter-label">分析日期</label>
          <input
            type="date"
            v-model="filters.analyzeDate"
            class="filter-date"
          />
        </div>
      </div>
    </div>

    <!-- 分析任务状态 -->
    <div v-if="currentTask" class="task-status-card">
      <div class="task-header">
        <div class="task-title">
          <span class="task-icon">🚀</span>
          分析任务进行中
        </div>
        <div class="task-id">#{{ currentTask.id.slice(0, 8) }}...</div>
      </div>
      <div class="task-progress">
        <div class="progress-info">
          <div class="progress-text">
            {{ currentTask.processed_articles }} / {{ currentTask.total_articles }} 篇文章
          </div>
          <div class="progress-stats">
            <span class="success-count">✅ {{ currentTask.success_count }}</span>
            <span class="failed-count">❌ {{ currentTask.failed_count }}</span>
          </div>
        </div>
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            :style="{ width: `${progressPercentage}%` }"
          ></div>
        </div>
      </div>
      <div class="task-status-badge" :class="currentTask.status">
        {{ getStatusText(currentTask.status) }}
      </div>
    </div>

    <!-- 分析结果列表 -->
    <div class="results-content">
      <div v-if="analyzedNews.length === 0" class="empty-state">
        <div class="empty-icon">📊</div>
        <div class="empty-title">暂无分析结果</div>
        <div class="empty-description">
          请先在文章页面选择文章并开始分析，AI将为您提取关键信息
        </div>
        <button class="btn btn-primary" @click="switchToArticles">
          前往文章页面
        </button>
      </div>

      <div v-else class="results-list">
        <div v-for="news in filteredNews" :key="news.id" class="result-item" :class="{ 'selected': selectedNews.includes(news.id) }">
          <div class="item-checkbox">
            <input
              type="checkbox"
              :checked="selectedNews.includes(news.id)"
              @change="toggleNewsSelection(news.id)"
              class="news-checkbox"
            />
          </div>

          <div class="item-content">
            <div class="item-header">
              <h3 class="item-title">
                <a :href="news.original_url" target="_blank" class="title-link">{{ news.title }}</a>
              </h3>
              <div class="item-time">
                <span class="time-icon">🕒</span>
                {{ formatTime(news.analyzed_at) }}
              </div>
            </div>

            <div class="item-tags">
              <div class="tag-group">
                <span class="tag-label">行业:</span>
                <div class="tag-list">
                  <span v-for="tag in news.industry_type.split(',')" :key="tag" class="tag industry-tag">
                    {{ tag.trim() }}
                  </span>
                </div>
              </div>
              <div class="tag-group">
                <span class="tag-label">类型:</span>
                <div class="tag-list">
                  <span v-for="tag in news.news_type.split(',')" :key="tag" class="tag news-tag">
                    {{ tag.trim() }}
                  </span>
                </div>
              </div>
            </div>

            <div class="item-summary">
              <p class="summary-text">{{ news.summary }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 导出模态框 -->
    <div v-if="showExportModal" class="modal-overlay" @click="closeExportModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3 class="modal-title">📤 导出分析结果</h3>
          <button class="close-btn" @click="closeExportModal">✕</button>
        </div>

        <div class="modal-body">
          <div class="export-options">
            <div class="option-group">
              <label class="option-label">导出范围:</label>
              <div class="radio-group">
                <label class="radio-option">
                  <input type="radio" v-model="exportOptions.scope" value="all" />
                  <span>所有结果 ({{ filteredNews.length }} 条)</span>
                </label>
                <label class="radio-option">
                  <input type="radio" v-model="exportOptions.scope" value="selected" :disabled="selectedNews.length === 0" />
                  <span>选中结果 ({{ selectedNews.length }} 条)</span>
                </label>
              </div>
            </div>

            <div class="option-group">
              <label class="option-label">导出格式:</label>
              <div class="radio-group">
                <label class="radio-option">
                  <input type="radio" v-model="exportOptions.format" value="markdown" />
                  <span>Markdown (.md)</span>
                </label>
                <label class="radio-option">
                  <input type="radio" v-model="exportOptions.format" value="clipboard" />
                  <span>复制到剪切板</span>
                </label>
              </div>
            </div>

            <div class="option-group">
              <label class="option-label">包含字段:</label>
              <div class="checkbox-group">
                <label class="checkbox-option">
                  <input type="checkbox" v-model="exportOptions.includeTitle" />
                  <span>标题</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" v-model="exportOptions.includeURL" />
                  <span>原文链接</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" v-model="exportOptions.includeIndustry" />
                  <span>行业类型</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" v-model="exportOptions.includeNewsType" />
                  <span>新闻类型</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" v-model="exportOptions.includeSummary" />
                  <span>内容摘要</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" v-model="exportOptions.includeTime" />
                  <span>分析时间</span>
                </label>
              </div>
            </div>

            <div class="option-group">
              <label class="option-label">分组方式:</label>
              <select v-model="exportOptions.groupBy" class="group-select">
                <option value="none">不分组</option>
                <option value="industry">按行业分组</option>
                <option value="newsType">按新闻类型分组</option>
                <option value="date">按日期分组</option>
              </select>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-secondary" @click="closeExportModal">取消</button>
          <button class="btn btn-primary" @click="handleExport" :disabled="!canExport">
            {{ exportOptions.format === 'markdown' ? '生成MD文件' : '复制到剪切板' }}
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- 实时日志面板 -->
  <div v-if="showLogsPanel" class="logs-panel">
    <div class="logs-header">
      <div class="logs-title">
        <span class="logs-icon">📋</span>
        实时分析日志
        <span class="logs-count">({{ filteredLogs.length }} 条)</span>
      </div>
      <div class="logs-controls">
        <div class="log-filter">
          <select v-model="logFilter" class="filter-select">
            <option value="all">全部</option>
            <option value="info">信息 ({{ logStats.info }})</option>
            <option value="warn">警告 ({{ logStats.warn }})</option>
            <option value="error">错误 ({{ logStats.error }})</option>
          </select>
        </div>
        <label class="auto-scroll-label">
          <input type="checkbox" v-model="autoScroll" />
          自动滚动
        </label>
        <button class="btn btn-sm btn-outline" @click="clearLogs">清空日志</button>
        <button class="btn btn-sm" @click="showLogsPanel = false">✕</button>
      </div>
    </div>
    <div class="logs-content" ref="logsContent">
      <div v-if="filteredLogs.length === 0" class="logs-empty">
        暂无日志记录
      </div>
      <div
        v-for="log in filteredLogs"
        :key="log.id"
        class="log-item"
        :class="`log-${log.level}`"
      >
        <div class="log-time">
          {{ formatLogTime(log.timestamp) }}
        </div>
        <div class="log-level" :class="`level-${log.level}`">
          {{ log.level.toUpperCase() }}
        </div>
        <div class="log-message">
          {{ log.message }}
          <div v-if="log.context && log.context.article_title" class="log-context">
            📄 {{ log.context.article_title }}
          </div>
          <div v-if="log.context && log.context.current_step" class="log-progress">
            <span v-if="log.context.progress !== undefined && log.context.total">
              {{ log.context.progress }} / {{ log.context.total }} -
            </span>
            {{ log.context.current_step }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import type { AnalysisTask, AnalyzedNews, IndustryType, NewsType, AnalysisLog } from '../types'
import { formatIsoString, isoStringToUtcTimestamp } from '../utils/timeUtils'

const currentTask = ref<AnalysisTask | null>(null)
const analyzedNews = ref<AnalyzedNews[]>([])
const filteredNews = ref<AnalyzedNews[]>([])
const selectedNews = ref<string[]>([])

// 导出相关状态
const showExportModal = ref(false)
const exportOptions = ref({
  scope: 'selected', // all | selected - 默认为选中
  format: 'markdown', // markdown | clipboard
  includeTitle: true,
  includeURL: true,
  includeIndustry: false,
  includeNewsType: false,
  includeSummary: true,
  includeTime: false,
  groupBy: 'none' // none | industry | newsType | date
})

// 实时日志相关状态
const showLogsPanel = ref(false)
const analysisLogs = ref<AnalysisLog[]>([])
const logFilter = ref<'all' | 'info' | 'warn' | 'error'>('all')
const autoScroll = ref(true)

// 筛选器状态
const filters = ref({
  industryType: '',
  newsType: '',
  analyzeDate: ''
})

// 获取唯一的行业类型和新闻类型
const uniqueIndustryTypes = computed(() => {
  const allTags = new Set<string>()
  ;(analyzedNews.value || []).forEach(news => {
    // 分割逗号分隔的标签，并去除前后空格
    const tags = news.industry_type.split(',').map(tag => tag.trim()).filter(tag => tag)
    tags.forEach(tag => allTags.add(tag))
  })
  return Array.from(allTags).sort()
})

const uniqueNewsTypes = computed(() => {
  const allTags = new Set<string>()
  ;(analyzedNews.value || []).forEach(news => {
    // 分割逗号分隔的标签，并去除前后空格
    const tags = news.news_type.split(',').map(tag => tag.trim()).filter(tag => tag)
    tags.forEach(tag => allTags.add(tag))
  })
  return Array.from(allTags).sort()
})

const progressPercentage = computed(() => {
  if (!currentTask.value) return 0
  return (currentTask.value.processed_articles / currentTask.value.total_articles) * 100
})

const allSelected = computed(() => {
  return filteredNews.value.length > 0 && selectedNews.value.length === filteredNews.value.length
})

// 计算当前筛选状态的文本
const filterStatusText = computed(() => {
  const hasFilters = filters.value.industryType || filters.value.newsType || filters.value.analyzeDate
  if (!hasFilters) {
    return `显示全部 ${analyzedNews.value.length} 条结果`
  }
  return `已筛选出 ${filteredNews.value.length} 条结果（共 ${analyzedNews.value.length} 条）`
})

const canExport = computed(() => {
  const hasSelectedItems = exportOptions.value.scope === 'selected'
    ? selectedNews.value.length > 0
    : filteredNews.value.length > 0

  const hasSelectedFields = Object.values(exportOptions.value).some((value, key) => {
    // 排除非字段选项
    return !['scope', 'format', 'groupBy'].includes(key) && value === true
  })

  // 实时进度更新
let progressUpdateInterval: NodeJS.Timeout | null = null

  return hasSelectedItems && hasSelectedFields
})

// 增强进度更新函数，添加日志记录
const startProgressUpdate = () => {
  stopProgressUpdate()
  addLog('info', '开始监控分析任务进度')

  progressUpdateInterval = setInterval(async () => {
    if (currentTask.value && currentTask.value.status === 'running') {
      try {
        const task = await invoke<AnalysisTask>('get_analysis_task', { taskId: currentTask.value.id })
        currentTask.value = task
        // 如果任务完成，停止更新并加载结果
        if (task.status === 'completed' || task.status === 'failed') {
          stopProgressUpdate()
          await loadResults()
        }
      } catch (error) {
        console.error('更新任务状态失败:', error)
        // 如果任务不存在，停止更新
        if ((error as string).toString().includes('分析任务不存在')) {
          stopProgressUpdate()
        }
      }
    }
  }, 3000) // 每3秒更新一次
}

const stopProgressUpdate = () => {
  if (progressUpdateInterval) {
    clearInterval(progressUpdateInterval)
    progressUpdateInterval = null
  }
}

// 监听当前任务变化
watch(currentTask, (newTask) => {
  if (newTask && newTask.status === 'running') {
    startProgressUpdate()
  } else {
    stopProgressUpdate()
  }
})

onMounted(async () => {
  // 直接加载任务和结果，不再需要类型数据
  await loadLatestTask()
  await loadResults()

  // 监听来自文章页面的页面切换事件
  window.addEventListener('switchToAnalysis', handleSwitchToAnalysis)
})

onUnmounted(() => {
  stopProgressUpdate()
  // 移除事件监听
  window.removeEventListener('switchToAnalysis', handleSwitchToAnalysis)
})

// 处理从文章页面跳转过来的事件
const handleSwitchToAnalysis = async (event: CustomEvent) => {
  console.log('接收到切换到分析页面的事件:', event.detail)

  const { taskId, articlesCount } = event.detail

  try {
    // 立即刷新任务状态
    await loadLatestTask()

    // 显示任务已开始的通知
    showNotification(`分析任务已启动！正在分析 ${articlesCount} 篇文章，请查看下方进度`, 'success')

    // 开始更新进度
    if (currentTask.value && currentTask.value.status === 'running') {
      startProgressUpdate()
    }
  } catch (error) {
    console.error('处理页面切换事件失败:', error)
  }
}

// 显示通知的统一方法
const showNotification = (message: string, type: 'info' | 'success' | 'warning' | 'error' = 'info') => {
  try {
    if (window.app && window.app.$notify) {
      window.app.$notify[type](message, { timeout: 3000 })
    } else {
      // 使用 alert 作为备用方案，但类型化为通知
      const typeIcons = {
        info: 'ℹ️',
        success: '✅',
        warning: '⚠️',
        error: '❌'
      }
      alert(`${typeIcons[type]} ${message}`)
    }
  } catch (e) {
    console.error('显示通知失败:', e)
    alert(message)
  }
}

const loadLatestTask = async () => {
  try {
    // 获取所有分析任务，取最新的一个
    const tasks = await invoke<AnalysisTask[]>('get_analysis_tasks')
    const latestTask = tasks.length > 0 ? tasks[0] : null
    currentTask.value = latestTask
    if (latestTask) {
      // 如果任务还在运行中，定期更新状态
      if (latestTask.status === 'running') {
        setTimeout(loadLatestTask, 2000)
      }
    }
  } catch (error) {
    console.error('获取分析任务失败:', error)
  }
}

const loadResults = async () => {
  try {
    console.log('开始加载所有分析结果...')
    // 获取所有分析结果，而不仅仅是当前任务的
    const results = await invoke<AnalyzedNews[]>('get_all_analyzed_news', {
      limit: 500
    })
    console.log('获取到的分析结果数量:', results.length)
    analyzedNews.value = results
  } catch (error) {
    console.error('获取分析结果失败:', error)
    alert('获取分析结果失败: ' + error)
  }
}

const refreshResults = () => {
  loadLatestTask()
  loadResults()
}

const clearResults = async () => {
  if (!currentTask.value) return

  try {
    await invoke('clear_all_analyzed_news')
    analyzedNews.value = []
    alert('分析结果已清空')
  } catch (error) {
    console.error('清空分析结果失败:', error)
    alert('清空失败: ' + error)
  }
}


const getStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    pending: '等待中',
    running: '分析中',
    completed: '已完成',
    failed: '失败'
  }
  return statusMap[status] || status
}

const deleteNews = async (newsId: string) => {
  if (!confirm('确定要删除这条分析结果吗？')) {
    return
  }

  try {
    await invoke('delete_analyzed_news', { newsId })
    // 从本地数组中移除该条目
    const index = analyzedNews.value.findIndex(news => news.id === newsId)
    if (index > -1) {
      analyzedNews.value.splice(index, 1)
    }
    alert('分析结果已删除')
  } catch (error) {
    console.error('删除分析结果失败:', error)
    alert('删除失败: ' + error)
  }
}

const toggleSelectAll = () => {
  if (allSelected.value) {
    // 取消全选 - 只清除筛选后结果的选中状态
    const filteredIds = filteredNews.value.map(news => news.id)
    selectedNews.value = selectedNews.value.filter(id => !filteredIds.includes(id))
  } else {
    // 全选选出筛选后的结果
    const filteredIds = filteredNews.value.map(news => news.id)
    // 合并已选中的其他项和当前筛选结果
    const newSelected = [...new Set([...selectedNews.value, ...filteredIds])]
    selectedNews.value = newSelected
  }
}

const deleteSelected = async () => {
  if (selectedNews.value.length === 0) {
    return
  }

  if (!confirm(`确定要删除选中的 ${selectedNews.value.length} 条分析结果吗？`)) {
    return
  }

  try {
    await invoke('delete_multiple_analyzed_news', { newsIds: selectedNews.value })
    
    // 从本地数组中移除已删除的条目
    analyzedNews.value = analyzedNews.value.filter(news => !selectedNews.value.includes(news.id))
    
    // 清空选中状态
    selectedNews.value = []
    
    alert(`成功删除分析结果`)
  } catch (error) {
    console.error('批量删除分析结果失败:', error)
    alert('删除失败: ' + error)
  }
}

const toggleNewsSelection = (newsId: string) => {
  const index = selectedNews.value.indexOf(newsId)
  if (index > -1) {
    // 取消选中
    selectedNews.value.splice(index, 1)
  } else {
    // 选中
    selectedNews.value.push(newsId)
  }
}

// 筛选方法
const applyFilters = () => {
  filteredNews.value = analyzedNews.value.filter(news => {
    // 行业类型筛选 - 检查是否包含选中的单个标签
    if (filters.value.industryType) {
      const industryTags = news.industry_type.split(',').map(tag => tag.trim()).filter(tag => tag)
      if (!industryTags.includes(filters.value.industryType)) {
        return false
      }
    }

    // 新闻类型筛选 - 检查是否包含选中的单个标签
    if (filters.value.newsType) {
      const newsTags = news.news_type.split(',').map(tag => tag.trim()).filter(tag => tag)
      if (!newsTags.includes(filters.value.newsType)) {
        return false
      }
    }

    // 完成时间筛选 - 使用Date对象比较而不是字符串比较
    if (filters.value.analyzeDate) {
      const newsDate = new Date(news.analyzed_at)
      const filterDate = new Date(filters.value.analyzeDate)
      
      // 比较日期部分，忽略时间部分
      if (newsDate.getFullYear() !== filterDate.getFullYear() ||
          newsDate.getMonth() !== filterDate.getMonth() ||
          newsDate.getDate() !== filterDate.getDate()) {
        return false
      }
    }

    return true
  })
}

// 清除筛选
const clearFilters = () => {
  filters.value = {
    industryType: '',
    newsType: '',
    analyzeDate: ''
  }
  filteredNews.value = [...analyzedNews.value]
}

// 格式化时间
const formatTime = (timeStr: string) => {
  // 使用统一的时间处理工具函数
  return formatIsoString(timeStr, 'relative')
}

// 切换到文章页面
const switchToArticles = () => {
  console.log('切换到文章页面')
}

// 导出相关方法
const closeExportModal = () => {
  showExportModal.value = false
}

const getNewsForExport = () => {
  return exportOptions.value.scope === 'selected'
    ? analyzedNews.value.filter(news => selectedNews.value.includes(news.id))
    : filteredNews.value
}

const groupNews = (newsList: AnalyzedNews[]) => {
  const groups: Record<string, AnalyzedNews[]> = {}

  switch (exportOptions.value.groupBy) {
    case 'industry':
      newsList.forEach(news => {
        const industries = news.industry_type.split(',').map(tag => tag.trim()).filter(tag => tag)
        const key = industries.length > 0 ? industries[0] : '未分类'
        if (!groups[key]) groups[key] = []
        groups[key].push(news)
      })
      break
    case 'newsType':
      newsList.forEach(news => {
        const types = news.news_type.split(',').map(tag => tag.trim()).filter(tag => tag)
        const key = types.length > 0 ? types[0] : '未分类'
        if (!groups[key]) groups[key] = []
        groups[key].push(news)
      })
      break
    case 'date':
      newsList.forEach(news => {
        const date = new Date(news.analyzed_at).toLocaleDateString('zh-CN')
        if (!groups[date]) groups[date] = []
        groups[date].push(news)
      })
      break
    default:
      groups['全部'] = newsList
  }

  return groups
}

const formatNewsItem = (news: AnalyzedNews, isMarkdown: boolean = false) => {
  const lines = []

  if (exportOptions.value.includeTitle) {
    if (isMarkdown && exportOptions.value.includeURL) {
      // MD格式且包含链接时，将链接作为标题的超链接
      lines.push(`**[${news.title}](${news.original_url})**`)
    } else {
      // 其他情况或MD格式不包含链接时，只显示标题
      lines.push(`**${news.title}**`)
    }
  }

  // MD格式且包含链接时，不单独显示链接字段
  if (exportOptions.value.includeURL && !(isMarkdown && exportOptions.value.includeTitle)) {
    lines.push(`${news.original_url}`)
  }

  if (exportOptions.value.includeIndustry) {
    lines.push(`${news.industry_type}`)
  }

  if (exportOptions.value.includeNewsType) {
    lines.push(`${news.news_type}`)
  }

  if (exportOptions.value.includeSummary) {
    lines.push(`${news.summary}`)
  }

  if (exportOptions.value.includeTime) {
    const timeStr = formatTime(news.analyzed_at)
    lines.push(`${timeStr}`)
  }

  lines.push('') // 空行分隔

  return lines.join('\n')
}

const generateMarkdownContent = () => {
  const newsList = getNewsForExport()
  const groups = groupNews(newsList)

  const lines = []

  if (exportOptions.value.groupBy === 'none') {
    groups['全部'].forEach(news => {
      lines.push(formatNewsItem(news, true)) // MD格式，传递 isMarkdown = true
    })
  } else {
    Object.entries(groups).forEach(([groupName, groupItems]) => {
      lines.push(`## ${groupName} (${groupItems.length} 条)`)
      lines.push('')
      lines.push(groupItems.map(news => formatNewsItem(news, true)).join(''))
    })
  }

  return lines.join('\n').trim()
}

const generateClipboardContent = () => {
  const newsList = getNewsForExport()
  const groups = groupNews(newsList)

  const lines = []

  if (exportOptions.value.groupBy === 'none') {
    groups['全部'].forEach(news => {
      lines.push(formatNewsItem(news, false)) // 剪切板格式，传递 isMarkdown = false
    })
  } else {
    Object.entries(groups).forEach(([groupName, groupItems]) => {
      lines.push(`${groupName} (${groupItems.length} 条)`)
      lines.push('')
      lines.push(groupItems.map(news => formatNewsItem(news, false)).join(''))
    })
  }

  return lines.join('\n').trim()
}

const handleExport = async () => {
  try {
    const content = exportOptions.value.format === 'markdown'
      ? generateMarkdownContent()
      : generateClipboardContent()

    if (exportOptions.value.format === 'markdown') {
      // 使用Tauri的文件保存对话框
      const fileName = `新闻分析结果_${new Date().toISOString().split('T')[0]}.md`

      try {
        const filePath = await save({
          title: '保存分析结果',
          defaultPath: fileName,
          filters: [
            {
              name: 'Markdown文件',
              extensions: ['md']
            }
          ]
        })

        if (filePath) {
          await writeTextFile(filePath, content)
          alert('MD文件已保存成功！')
        }
      } catch (saveError: any) {
        if (saveError.message?.includes('User cancelled')) {
          // 用户取消了保存，不显示错误
          return
        }
        throw saveError
      }
    } else {
      // 复制到剪切板
      await navigator.clipboard.writeText(content)
      alert('内容已复制到剪切板')
    }

    closeExportModal()
  } catch (error) {
    console.error('导出失败:', error)
    alert('导出失败: ' + error)
  }
}

// 监听原始数据变化，自动更新筛选结果
watch(analyzedNews, () => {
  filteredNews.value = [...analyzedNews.value]
}, { immediate: true })

// 监听筛选条件变化
watch(filters, () => {
  applyFilters()
}, { deep: true })

// 筛选显示的日志
const filteredLogs = computed(() => {
  if (logFilter.value === 'all') {
    return analysisLogs.value
  }
  return analysisLogs.value.filter(log => log.level === logFilter.value)
})

// 日志统计
const logStats = computed(() => {
  const stats = { info: 0, warn: 0, error: 0, debug: 0 }
  analysisLogs.value.forEach(log => {
    stats[log.level]++
  })
  return stats
})

// 添加日志（模拟后端日志推送）
const addLog = (level: AnalysisLog['level'], message: string, context?: AnalysisLog['context']) => {
  const log: AnalysisLog = {
    id: Date.now().toString(),
    timestamp: new Date().toISOString(),
    level,
    message,
    task_id: currentTask.value?.id || '',
    context
  }
  analysisLogs.value.push(log)

  // 限制日志数量，避免内存溢出
  if (analysisLogs.value.length > 1000) {
    analysisLogs.value = analysisLogs.value.slice(-500)
  }

  // 自动滚动到底部
  if (autoScroll.value) {
    nextTick(() => {
      scrollLogsToBottom()
    })
  }
}

// 滚动到日志底部
const scrollLogsToBottom = () => {
  const logsContainer = document.querySelector('.logs-content')
  if (logsContainer) {
    logsContainer.scrollTop = logsContainer.scrollHeight
  }
}

// 清空日志
const clearLogs = () => {
  analysisLogs.value = []
}

// 格式化日志时间
const formatLogTime = (timestamp: string) => {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3
  })
}
</script>

<style scoped>
.analysis-results-page {
  background: transparent;
  padding: 0;
  min-height: 100vh;
  position: relative;
  z-index: 1;
}

/* 页面头部 */
.page-header {
  background: white;
  border-radius: 12px;
  padding: 20px 24px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #f0f0f0;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 24px;
}

.header-text {
  flex: 1;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 4px;
}

.page-subtitle {
  font-size: 14px;
  color: #6c757d;
  font-weight: 500;
}

.header-stats {
  display: flex;
  gap: 12px;
}

.stat-card {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 12px 16px;
  text-align: center;
  min-width: 60px;
  border: 1px solid #e9ecef;
}

.stat-number {
  font-size: 18px;
  font-weight: 600;
  color: #495057;
  margin-bottom: 2px;
}

.stat-label {
  font-size: 11px;
  color: #6c757d;
  font-weight: 500;
}

.header-actions {
  display: flex;
  gap: 12px;
}

/* 按钮样式 */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-icon {
  font-size: 14px;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover {
  background: #5a6fd8;
}

.btn-secondary {
  background: white;
  color: #6c757d;
  border: 1px solid #e9ecef;
}

.btn-secondary:hover {
  background: #f8f9fa;
  border-color: #dee2e6;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover {
  background: #c82333;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

.btn-outline {
  background: transparent;
  color: #6c757d;
  border: 1px solid #e9ecef;
}

.btn-outline:hover {
  background: #f8f9fa;
  color: #495057;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none !important;
}

/* 筛选器样式 */
.filters-section {
  background: white;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #f0f0f0;
}

.filters-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 12px;
}

.filter-status-text {
  font-size: 13px;
  color: #6c757d;
  background: #f8f9fa;
  padding: 6px 12px;
  border-radius: 20px;
  font-weight: 500;
}

.filters-title {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
  display: flex;
  align-items: center;
  gap: 6px;
}

.filters-actions {
  display: flex;
  gap: 8px;
}

.filter-row {
  display: flex;
  align-items: flex-end;
  gap: 20px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 180px;
}

.filter-label {
  font-size: 13px;
  font-weight: 600;
  color: #495057;
  margin-bottom: 4px;
}

.filter-select {
  padding: 6px 8px;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  font-size: 13px;
  background: white;
  min-width: 120px;
  transition: all 0.2s ease;
}

.filter-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
}

.filter-date {
  padding: 6px 8px;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  font-size: 13px;
  min-width: 120px;
  transition: all 0.2s ease;
}

.filter-date:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
}

/* 任务状态卡片 */
.task-status-card {
  background: white;
  border: 1px solid #e9ecef;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.task-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  color: #2c3e50;
}

.task-icon {
  font-size: 16px;
}

.task-id {
  font-size: 11px;
  color: #6c757d;
  font-family: 'Courier New', monospace;
  background: #f8f9fa;
  padding: 2px 6px;
  border-radius: 4px;
}

.task-progress {
  margin-bottom: 16px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.progress-text {
  font-size: 14px;
  color: #495057;
  font-weight: 500;
}

.progress-stats {
  display: flex;
  gap: 12px;
}

.success-count {
  color: #28a745;
  font-weight: 600;
}

.failed-count {
  color: #dc3545;
  font-weight: 600;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: #e9ecef;
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress-fill {
  height: 100%;
  background: #667eea;
  transition: width 0.3s ease;
}

.task-status-badge {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
  text-align: center;
  position: absolute;
  top: 16px;
  right: 20px;
}

.task-status-badge.pending {
  background: #ffc107;
  color: #856404;
}

.task-status-badge.running {
  background: #17a2b8;
  color: white;
}

.task-status-badge.completed {
  background: #28a745;
  color: white;
}

.task-status-badge.failed {
  background: #dc3545;
  color: white;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
 50% {
    opacity: 0.7;
  }
}

/* 结果内容区域 */
.results-content {
  min-height: 400px;
  position: relative;
  z-index: 1;
}

.empty-state {
  text-align: center;
  padding: 80px 20px;
  color: #6c757d;
  position: relative;
  z-index: 1;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 24px;
  opacity: 0.3;
  filter: grayscale(100%);
}

.empty-title {
  font-size: 24px;
  margin-bottom: 12px;
  font-weight: 600;
  color: #495057;
}

.empty-description {
  font-size: 16px;
  color: #6c757d;
  max-width: 400px;
  margin: 0 auto;
  line-height: 1.6;
  margin-bottom: 32px;
}

/* 结果列表布局 */
.results-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-item {
  background: white;
  border-radius: 8px;
  border: 1px solid #e9ecef;
  padding: 16px 20px;
  display: flex;
  gap: 16px;
  transition: all 0.2s ease;
}

.result-item:hover {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.result-item.selected {
  border: 2px solid #667eea;
  background: rgba(102, 126, 234, 0.05);
}

.item-checkbox {
  display: flex;
  align-items: flex-start;
  padding-top: 4px;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 12px;
}

.item-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  line-height: 1.4;
  flex: 1;
}

.item-tags {
  display: flex;
  gap: 20px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.news-checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: #667eea;
}

.title-link {
  color: #667eea;
  text-decoration: none;
  transition: color 0.3s ease;
  word-break: break-word;
}

.title-link:hover {
  color: #5a6fd8;
  text-decoration: underline;
}

.time-icon {
  font-size: 14px;
}

.item-time {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #6c757d;
  white-space: nowrap;
  flex-shrink: 0;
}

/* 共用样式 */
.tag-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tag-label {
  font-size: 12px;
  font-weight: 600;
  color: #495057;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.industry-tag {
  background: #e3f2fd;
  color: #1976d2;
}

.news-tag {
  background: #f3e5f5;
  color: #7b1fa2;
}

.tag:hover {
  background: #e8eaf6;
}

.item-summary {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 12px;
}

.summary-text {
  font-size: 14px;
  line-height: 1.6;
  color: #495057;
  margin: 0;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    gap: 20px;
    align-items: stretch;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .filter-row {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
  }

  .result-item {
    flex-direction: column;
    gap: 12px;
  }

  .item-header {
    flex-direction: column;
    gap: 8px;
  }

  .item-tags {
    flex-direction: column;
    gap: 12px;
  }

  .item-time {
    align-self: flex-end;
  }
}

@media (max-width: 480px) {
  .page-header {
    padding: 20px;
  }

  .result-item {
    padding: 16px;
  }

  .item-header {
    gap: 12px;
  }

  .item-title {
    font-size: 15px;
  }

  .item-summary {
    padding: 10px;
  }

  .summary-text {
    font-size: 13px;
  }
}

/* 动画效果 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.result-item {
  animation: fadeInUp 0.6s ease-out;
}

.result-item:nth-child(even) {
  animation-delay: 0.1s;
}

.result-item:nth-child(odd) {
  animation-delay: 0.2s;
}

/* 导出模态框样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.modal-content {
  background: white;
  border-radius: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
  animation: modalFadeIn 0.3s ease-out;
}

@keyframes modalFadeIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 24px 20px 24px;
  border-bottom: 1px solid #e9ecef;
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: #2c3e50;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  color: #6c757d;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: #f8f9fa;
  color: #495057;
}

.modal-body {
  padding: 24px;
}

.modal-footer {
  padding: 20px 24px 24px 24px;
  border-top: 1px solid #e9ecef;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* 导出选项样式 */
.export-options {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.option-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.option-label {
  font-size: 14px;
  font-weight: 600;
  color: #495057;
  margin-bottom: 8px;
}

.radio-group, .checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.radio-option, .checkbox-option {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 6px 8px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.radio-option:hover, .checkbox-option:hover {
  background: #f8f9fa;
}

.radio-option input[type="radio"],
.checkbox-option input[type="checkbox"] {
  margin: 0;
  cursor: pointer;
}

.radio-option span, .checkbox-option span {
  font-size: 14px;
  color: #495057;
}

.group-select {
  padding: 8px 12px;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  font-size: 14px;
  background: white;
  transition: all 0.2s ease;
  min-width: 200px;
}

.group-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
}

/* 响应式设计 - 模态框 */
@media (max-width: 768px) {
  .modal-content {
    width: 95%;
    max-height: 90vh;
  }

  .modal-header, .modal-body, .modal-footer {
    padding: 20px;
  }

  .export-options {
    gap: 20px;
  }

  .radio-group, .checkbox-group {
    gap: 10px;
  }

  .radio-option, .checkbox-option {
    padding: 10px;
  }
}

@media (max-width: 480px) {
  .modal-content {
    width: 98%;
    max-height: 95vh;
  }

  .modal-header, .modal-body, .modal-footer {
    padding: 16px;
  }

  .export-options {
    gap: 16px;
  }

  .group-select {
    min-width: 100%;
  }

  .modal-footer {
    flex-direction: column;
    gap: 8px;
  }

  .btn {
    width: 100%;
    justify-content: center;
  }
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, #5a6fd8, #6a4190);
}

/* 实时日志面板样式 */
.logs-panel {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 600px;
  max-width: 90vw;
  height: 400px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
  border: 1px solid #e9ecef;
  display: flex;
  flex-direction: column;
  z-index: 1000;
  animation: slideInUp 0.3s ease-out;
}

@keyframes slideInUp {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e9ecef;
  background: #f8f9fa;
  border-radius: 12px 12px 0 0;
  flex-wrap: wrap;
  gap: 12px;
}

.logs-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
}

.logs-count {
  color: #6c757d;
  font-size: 12px;
  font-weight: normal;
}

.logs-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.log-filter .filter-select {
  font-size: 12px;
  padding: 4px 8px;
  border: 1px solid #e9ecef;
  border-radius: 4px;
  min-width: 120px;
}

.auto-scroll-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #495057;
  cursor: pointer;
}

.auto-scroll-label input[type="checkbox"] {
  margin: 0;
  cursor: pointer;
}

.logs-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  background: #fafbfc;
}

.logs-empty {
  text-align: center;
  color: #6c757d;
  padding: 40px 20px;
  font-size: 14px;
}

.log-item {
  display: flex;
  gap: 12px;
  padding: 8px 20px;
  font-size: 13px;
  border-left: 3px solid transparent;
  transition: all 0.2s ease;
  align-items: flex-start;
  min-height: 40px;
}

.log-item:hover {
  background: rgba(0, 0, 0, 0.02);
}

.log-time {
  color: #6c757d;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  min-width: 90px;
  line-height: 1.6;
  flex-shrink: 0;
}

.log-level {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  min-width: 50px;
  text-align: center;
  line-height: 1.4;
  flex-shrink: 0;
}

.level-info {
  background: #e3f2fd;
  color: #1976d2;
}

.level-warn {
  background: #fff3cd;
  color: #856404;
}

.level-error {
  background: #f8d7da;
  color: #721c24;
}

.level-debug {
  background: #e2e3e5;
  color: #383d41;
}

.log-message {
  flex: 1;
  line-height: 1.6;
  color: #495057;
  word-break: break-word;
}

.log-context {
  font-size: 12px;
  color: #6c757d;
  margin-top: 4px;
  padding: 4px 8px;
  background: #f8f9fa;
  border-radius: 4px;
  border-left: 2px solid #667eea;
}

.log-progress {
  font-size: 12px;
  color: #28a745;
  margin-top: 4px;
  padding: 4px 8px;
  background: #d4edda;
  border-radius: 4px;
  font-weight: 500;
}

.log-info {
  border-left-color: #1976d2;
}

.log-warn {
  border-left-color: #856404;
}

.log-error {
  border-left-color: #721c24;
  background: rgba(248, 215, 218, 0.1);
}

.log-debug {
  border-left-color: #383d41;
}

/* 错误指示器 */
.error-indicator {
  background: #dc3545;
  color: white;
  border-radius: 50%;
  width: 18px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  margin-left: 6px;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.1);
  }
}

/* 响应式设计 - 日志面板 */
@media (max-width: 768px) {
  .logs-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    width: 100%;
    height: 60vh;
    border-radius: 12px 12px 0 0;
  }

  .logs-header {
    padding: 12px 16px;
    flex-wrap: wrap;
    gap: 8px;
  }

  .logs-title {
    font-size: 14px;
  }

  .logs-controls {
    gap: 8px;
    flex-wrap: wrap;
  }

  .log-item {
    padding: 6px 16px;
    font-size: 12px;
  }

  .log-time {
    min-width: 70px;
    font-size: 10px;
  }

  .log-level {
    min-width: 40px;
    font-size: 9px;
  }
}

@media (max-width: 480px) {
  .logs-panel {
    height: 70vh;
  }

  .logs-header {
    padding: 10px 12px;
  }

  .logs-title {
    font-size: 13px;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .log-item {
    flex-direction: column;
    gap: 6px;
    padding: 8px 12px;
  }

  .log-time {
    min-width: auto;
    order: 2;
  }

  .log-level {
    min-width: auto;
    order: 3;
  }

  .log-message {
    order: 1;
  }
}
</style>
